use core::fmt;
use core::str;

/// A DNS name.
#[derive(Debug, Clone)]
pub struct Name<'a> {
  pub(crate) buf: &'a [u8],
  pub(crate) start: usize,
}

impl<'a> Name<'a> {
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

pub(crate) struct Labels<'a> {
  buf: &'a [u8],
  buf_i: usize,
}

impl<'a> Iterator for Labels<'a> {
  type Item = &'a [u8];

  fn next(&mut self) -> Option<Self::Item> {
    loop {
      match read_label(self.buf, &mut self.buf_i) {
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

pub(crate) enum Label {
  Pointer(u16),
  Part(u8),
}

/// # TODO
///
/// Verify [CVE-2000-0333]/[VU#23495] is fully prevented in this function.
///
/// [CVE-2000-0333]: https://nvd.nist.gov/vuln/detail/CVE-2000-0333
/// [VU#23495]: https://www.kb.cert.org/vuls/id/23495/
pub(crate) fn read_name(buf: &[u8], i: &mut usize) -> bool {
  let global_start = *i;

  let mut iteration: u8 = 0;
  let mut len: u8 = 0;

  loop {
    let local_start = *i;

    match read_label(buf, i) {
      None => return false,
      Some(Label::Pointer(ptr)) => {
        // Stop following self-referencing pointer.
        if ptr as usize == global_start || ptr as usize == local_start {
          return false
        }

        *i = ptr as usize;
      },
      Some(Label::Part(part_len)) => {
        if part_len == 0 {
          return true
        }

        // Stop if maximum name length of 255 bytes is reached.
        len = if let Some(len) = len.checked_add(part_len) {
          len
        } else {
          return false
        };
      },
    }

    // Stop after a maximum 255 iterations.
    iteration = if let Some(iteration) = iteration.checked_add(1) {
      iteration
    } else {
      return false
    };
  }
}

const PTR_MASK: u8 = 0b11000000;
const LEN_MASK: u8 = !PTR_MASK;

// Return whether a label was read and whether it was a pointer or a normal name part.
fn read_label(buf: &[u8], i: &mut usize) -> Option<Label> {
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
