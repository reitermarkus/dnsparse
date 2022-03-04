use core::mem::size_of;

use crate::Error;

/// The kind of a DNS query.
///
/// According to [RFC 1035 Section 3.2.2](https://tools.ietf.org/rfc/rfc1035#section-3.2.2)
/// and [RFC 1035 Section 3.2.3](https://tools.ietf.org/rfc/rfc1035#section-3.2.3).
#[derive(Copy, Clone, Debug, PartialEq)]
#[repr(u16)]
pub enum QueryKind {
  A = 1,
  NS = 2,
  MD = 3,
  MF = 4,
  CNAME = 5,
  SOA = 6,
  MB = 7,
  MG = 8,
  MR = 9,
  NULL = 10,
  WKS = 11,
  PTR = 12,
  HINFO = 13,
  MINFO = 14,
  MX = 15,
  TXT = 16,
  AXFR = 252,
  MAILB = 253,
  MAILA = 254,
  ALL = 255,
  Reserved,
}

impl From<u16> for QueryKind {
  fn from(n: u16) -> Self {
    match n {
      1 => Self::A,
      2 => Self::NS,
      3 => Self::MD,
      4 => Self::MF,
      5 => Self::CNAME,
      6 => Self::SOA,
      7 => Self::MB,
      8 => Self::MG,
      9 => Self::MR,
      10 => Self::NULL,
      11 => Self::WKS,
      12 => Self::PTR,
      13 => Self::HINFO,
      14 => Self::MINFO,
      15 => Self::MX,
      16 => Self::TXT,
      252 => Self::AXFR,
      253 => Self::MAILA,
      254 => Self::MAILB,
      255 => Self::ALL,
      _ => Self::Reserved,
    }
  }
}

impl QueryKind {
  pub(crate) fn read(buf: &[u8], i: &mut usize) -> Result<Self, Error> {
    if *i + size_of::<QueryKind>() <= buf.len() {
      let query_kind = u16::from_be_bytes([buf[*i], buf[*i + 1]]);
      *i += size_of::<QueryKind>();

      return Ok(query_kind.into())
    }

    Err(Error::MessageTooShort)
  }

  pub(crate) fn to_be_bytes(self) -> [u8; 2] {
    (self as u16).to_be_bytes()
  }
}
