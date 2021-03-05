use core::fmt;
use core::str;

/// A DNS name.
#[derive(Debug, Clone)]
pub struct Name<'a> {
  pub(crate) buf: &'a [u8],
  pub(crate) start: usize,
}

impl<'a> Name<'a> {
  #[cfg(test)]
  pub(crate) fn from_bytes(bytes: &'a [u8]) -> Self {
    Self { buf: bytes, start: 0 }
  }

  pub(crate) fn create_pointer(&self, sub_name: &Name<'_>) -> Option<[u8; 2]> {
    let mut labels = self.labels();

    loop {
      let i = labels.buf_i;

      if labels.next().is_some() {
        if self.equal_from(i, sub_name) {
          let [ptr_1, ptr_2] = (i as u16).to_be_bytes();
          return Some([ptr_1 | PTR_MASK, ptr_2]);
        }

        continue;
      }

      break;
    }

    None
  }

  fn equal_from(&self, i: usize, sub_name: &Name<'_>) -> bool {
    let mut this = Labels { buf: self.buf, buf_i: i };
    let mut other = sub_name.labels();

    loop {
      match (this.next(), other.next()) {
        (Some(t), Some(o)) => if !t.as_str().eq_ignore_ascii_case(o.as_str()) {
          break;
        },
        (None, None) => return true,
        _ => break,
      }
    }

    false
  }

  pub(crate) fn read(buf: &'a [u8], i: &'_ mut usize) -> Option<Self> {
    let mut j = *i;
    let mut maximum = *i;
    let mut ptr = None;

    let mut len: u8 = 0;

    loop {
      match LabelType::read(buf, ptr.as_mut().unwrap_or(&mut j)) {
        None => break,
        Some(LabelType::Pointer(p)) => {
          let p = p as usize;

          // Pointers can only point to previous occurences.
          if p >= maximum {
            break
          }

          maximum = p;
          ptr = Some(p);
        },
        Some(LabelType::Part(part_len)) => {
          if part_len == 0 {
            let name = Name {
              buf,
              start: *i,
            };

            *i = j;
            return Some(name)
          }

          // Stop if maximum name length of 255 bytes is reached.
          if let Some(l) = len.checked_add(part_len) {
            len = l;
          } else {
            break
          };
        },
      }
    }

    None
  }

  pub(crate) fn split(&self) -> (Label<'a>, Option<Name<'a>>) {
    let mut labels = self.labels();

    let label = labels.next().unwrap();
    let name = labels.next().map(|l| Name { buf: l.buf, start: l.buf_i });

    (label, name)
  }

  pub(crate) fn labels(&self) -> Labels<'a> {
    Labels {
      buf: self.buf,
      buf_i: self.start,
    }
  }
}

impl fmt::Display for Name<'_> {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    let mut print_dot = false;

    for label in self.labels() {
      if print_dot {
        '.'.fmt(f)?;
      }

      label.as_str().fmt(f)?;

      print_dot = true;
    }

    Ok(())
  }
}

impl PartialEq<str> for Name<'_> {
  fn eq(&self, other: &str) -> bool {
    let mut other_i = 0;

    let other = other.as_bytes();

    for label in self.labels() {
      if other_i != 0 {
        if other.get(other_i) != Some(&b'.') {
          return false
        } else {
          other_i += 1;
        }
      }

      if let Some(substring) = other.get(other_i..(other_i + label.len())) {
        if !label.as_bytes().eq_ignore_ascii_case(substring) {
          return false
        }
      } else {
        return false
      }

      other_i += label.len();
    }

    other_i == other.len()
  }
}

#[derive(Debug, Clone)]
pub(crate) struct Labels<'a> {
  buf: &'a [u8],
  buf_i: usize,
}

impl<'a> Iterator for Labels<'a> {
  type Item = Label<'a>;

  fn next(&mut self) -> Option<Self::Item> {
    loop {
      match LabelType::read(self.buf, &mut self.buf_i) {
        None => return None,
        Some(LabelType::Pointer(ptr)) => {
          self.buf_i = ptr as usize;
          continue;
        },
        Some(LabelType::Part(len)) => {
          if len == 0 {
            return None
          }

          let label = Label {
            buf: self.buf,
            buf_i: self.buf_i - len as usize - 1,
          };

          return Some(label)
        },
      }
    }
  }
}

#[derive(Debug)]
pub(crate) struct Label<'a> {
  pub(crate) buf: &'a [u8],
  pub(crate) buf_i: usize,
}

impl<'a> Label<'a> {
  #[inline]
  pub fn as_bytes(&self) -> &[u8] {
    &self.buf[(self.buf_i + 1)..(self.buf_i + 1 + self.len())]
  }

  #[inline]
  pub fn as_str(&self) -> &str {
    unsafe { str::from_utf8_unchecked(self.as_bytes()) }
  }

  #[inline]
  pub fn len(&self) -> usize {
    self.buf[self.buf_i] as usize
  }
}

#[derive(Debug)]
pub(crate) enum LabelType {
  Pointer(u16),
  Part(u8),
}

const PTR_MASK: u8 = 0b11000000;
const LEN_MASK: u8 = !PTR_MASK;

impl LabelType {
  /// Return whether a label was read and whether it was a pointer or a normal name part.
  pub(crate) fn read(buf: &[u8], i: &mut usize) -> Option<Self> {
    if let Some(&ptr_or_len) = buf.get(*i) {
      // Check for pointer:
      // https://tools.ietf.org/rfc/rfc1035#section-4.1.4
      if ptr_or_len & PTR_MASK == PTR_MASK {
        if let Some(&ptr) = buf.get(*i + 1) {
          *i += 1 + 1;
          return Some(Self::Pointer(u16::from_be_bytes([ptr_or_len & LEN_MASK, ptr])))
        }
      } else {
        *i += 1 + (ptr_or_len & LEN_MASK) as usize;
        return Some(Self::Part(ptr_or_len & LEN_MASK))
      }
    }

    None
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_name_create_pointer() {
    let name = Name::from_bytes(&[1, b'a', 1, b'b', 1, b'c', 0]);
    let name_uppercase = Name::from_bytes(&[1, b'A', 1, b'B', 1, b'C', 0]);
    let sub_name = Name::from_bytes(&[1, b'b', 1, b'c', 0]);
    let sub_sub_name = Name::from_bytes(&[1, b'c', 0]);

    assert_eq!(name.create_pointer(&name), Some([0b11000000, 0]));
    assert_eq!(name_uppercase.create_pointer(&name), Some([0b11000000, 0]));
    assert_eq!(name.create_pointer(&name_uppercase), Some([0b11000000, 0]));
    assert_eq!(name.create_pointer(&sub_name), Some([0b11000000, 2]));
    assert_eq!(sub_name.create_pointer(&name), None);
    assert_eq!(sub_name.create_pointer(&sub_sub_name), Some([0b11000000, 2]));
    assert_eq!(name.create_pointer(&sub_sub_name), Some([0b11000000, 4]));
    assert_eq!(sub_sub_name.create_pointer(&name), None);
    assert_eq!(sub_sub_name.create_pointer(&sub_name), None);
  }
}
