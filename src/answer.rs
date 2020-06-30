use crate::{Name, QueryKind, QueryClass};

/// A DNS answer.
pub struct Answer<'a> {
  pub name: Name<'a>,
  pub kind: QueryKind,
  pub class: QueryClass,
  pub ttl: u32,
  pub rdata: &'a [u8],
}

impl<'a> Answer<'a> {
  pub fn name(&self) -> &Name<'a> {
    &self.name
  }

  pub fn kind(&self) -> &QueryKind {
    &self.kind
  }

  pub fn class(&self) -> &QueryClass {
    &self.class
  }

  pub fn ttl(&self) -> u32 {
    self.ttl
  }

  pub fn rdata(&self) -> &'a [u8] {
    &self.rdata
  }
}
