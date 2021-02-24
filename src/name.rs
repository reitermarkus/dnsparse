use core::fmt;
use core::str;

/// A DNS name.
#[derive(Debug, Clone)]
pub struct Name<'a> {
  pub(crate) buf: &'a [u8],
  pub(crate) start: usize,
}

impl<'a> Name<'a> {
  pub fn from_bytes(bytes: &'a [u8]) -> Self {
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
        (Some(t), Some(o)) => if !t.iter().zip(o.iter()).all(|(t, o)| t.eq_ignore_ascii_case(o)) {
          break;
        },
        (None, None) => return true,
        _ => break,
      }
    }

    false
  }

  pub(crate) fn read(buf: &[u8], i: &mut usize) -> bool {
    let global_start = *i;

    let mut iteration: u8 = 0;
    let mut len: u8 = 0;

    return read_rec(buf, i, global_start, &mut iteration, &mut len);

    fn read_rec(buf: &[u8], i: &mut usize, global_start: usize, iteration: &mut u8, len: &mut u8) -> bool {
      let rec_start = *i;

      loop {
        match Label::read(buf, i) {
          None => return false,
          Some(Label::Pointer(ptr)) => {
            let mut ptr = ptr as usize;

            // Stop following self-referencing pointer.
            if ptr == global_start || (ptr >= rec_start && ptr < *i) {
              return false
            }

            return read_rec(buf, &mut ptr, global_start, iteration, len);
          },
          Some(Label::Part(part_len)) => {
            if part_len == 0 {
              return true
            }

            // Stop if maximum name length of 255 bytes is reached.
            *len = if let Some(len) = len.checked_add(part_len) {
              len
            } else {
              return false
            };
          },
        }

        // Stop after a maximum 255 iterations.
        *iteration = if let Some(iteration) = iteration.checked_add(1) {
          iteration
        } else {
          return false
        };
      }
    }
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

      if let Ok(s) = str::from_utf8(label) {
        s.fmt(f)?;
      } else {
        return Err(fmt::Error)
      }

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
        if !label.eq_ignore_ascii_case(substring) {
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
  type Item = &'a [u8];

  fn next(&mut self) -> Option<Self::Item> {
    loop {
      match Label::read(self.buf, &mut self.buf_i) {
        None => return None,
        Some(Label::Pointer(ptr)) => {
          self.buf_i = ptr as usize;
          continue;
        },
        Some(Label::Part(len)) => {
          if len == 0 {
            return None
          }

          return Some(&self.buf[(self.buf_i - len as usize)..self.buf_i])
        },
      }
    }
  }
}

#[derive(Debug)]
pub(crate) enum Label {
  Pointer(u16),
  Part(u8),
}

const PTR_MASK: u8 = 0b11000000;
const LEN_MASK: u8 = !PTR_MASK;

impl Label {
  /// Return whether a label was read and whether it was a pointer or a normal name part.
  pub(crate) fn read(buf: &[u8], i: &mut usize) -> Option<Label> {
    if let Some(&ptr_or_len) = buf.get(*i) {
      // Check for pointer:
      // https://tools.ietf.org/rfc/rfc1035#section-4.1.4
      if ptr_or_len & PTR_MASK == PTR_MASK {
        if let Some(&ptr) = buf.get(*i + 1) {
          *i += 1 + 1;
          Some(Label::Pointer(u16::from_be_bytes([ptr_or_len & LEN_MASK, ptr])))
        } else {
          None
        }
      } else {
        *i += 1 + (ptr_or_len & LEN_MASK) as usize;
        Some(Label::Part(ptr_or_len & LEN_MASK))
      }
    } else {
      None
    }
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
