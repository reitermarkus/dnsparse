use core::mem::size_of;

/// The class of a DNS query.
///
/// According to [RFC 1035 Section 3.2.4](https://tools.ietf.org/rfc/rfc1035#section-3.2.4).
#[derive(Copy, Clone, Debug, PartialEq)]
#[repr(u16)]
pub enum QueryClass {
  /// Internet
  IN = 1,
  /// CSNET
  CS = 2,
  /// CHAOS
  CH = 3,
  /// Hesiod
  HS = 4,
  Reserved,
}

impl QueryClass {
  pub(crate) fn read(buf: &[u8], i: &mut usize) -> Option<Self> {
    if *i + size_of::<QueryClass>() <= buf.len() {
      let query_class = u16::from_be_bytes([buf[*i], buf[*i + 1]]);
      *i += size_of::<QueryClass>();

      return Some(query_class.into())
    }

    None
  }

  pub fn to_be_bytes(&self) -> [u8; 2] {
    (*self as u16).to_be_bytes()
  }
}

impl From<u16> for QueryClass {
  fn from(n: u16) -> Self {
    match n {
      1 => Self::IN,
      2 => Self::CS,
      3 => Self::CH,
      4 => Self::HS,
      _ => Self::Reserved,
    }
  }
}
