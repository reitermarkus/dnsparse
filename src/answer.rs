use core::mem::size_of;

use crate::{Error, Name, QueryKind, QueryClass};

/// A DNS answer.
#[derive(Debug)]
pub struct Answer<'a> {
  pub name: Name<'a>,
  pub kind: QueryKind,
  pub class: QueryClass,
  pub ttl: u32,
  pub rdata: &'a [u8],
}

fn read_ttl(buf: &[u8], i: &mut usize) -> Result<u32, Error> {
  if *i + size_of::<u32>() <= buf.len() {
    let ttl = u32::from_be_bytes([buf[*i], buf[*i + 1], buf[*i + 2], buf[*i + 3]]);
    *i += size_of::<u32>();
    return Ok(ttl)
  }

  Err(Error::MessageTooShort)
}

fn read_rdata<'a>(buf: &'a [u8], i: &'_ mut usize) -> Result<&'a [u8], Error> {
  if *i + size_of::<u16>() <= buf.len() {
    let rdata_len = u16::from_be_bytes([buf[*i], buf[*i + 1]]) as usize;
    let rdata_i = *i + size_of::<u16>();
    if rdata_i + rdata_len <= buf.len() {
      let rdata = &buf[rdata_i..(rdata_i + rdata_len)];
      *i = rdata_i + rdata_len;
      return Ok(rdata);
    }
  }

  Err(Error::MessageTooShort)
}

impl<'a> Answer<'a> {
  pub(crate) fn read(buf: &'a [u8], i: &'_ mut usize) -> Result<Self, Error> {
    let mut j = *i;
    let answer = Self {
      name:  Name::read(buf, &mut j)?,
      kind:  QueryKind::read(buf, &mut j)?,
      class: QueryClass::read(buf, &mut j)?,
      ttl:   read_ttl(buf, &mut j)?,
      rdata: read_rdata(buf, &mut j)?,
    };
    *i = j;

    Ok(answer)
  }

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
    self.rdata
  }
}

/// Iterator over [`Answer`](struct.Answer.html)s contained in a [`Message`](struct.Message.html).
#[derive(Debug)]
pub struct Answers<'a> {
  pub(crate) answer_count: usize,
  pub(crate) current_answer: usize,
  pub(crate) buf: &'a [u8],
  pub(crate) buf_i: usize,
}

impl<'a> Iterator for Answers<'a> {
  type Item = Answer<'a>;

  fn next(&mut self) -> Option<Self::Item> {
    if self.current_answer >= self.answer_count {
      return None
    }

    // `Answers` can only be created from a valid `Message`, so
    // reading an `Answer` should always succeed here.
    let answer = Answer::read(self.buf, &mut self.buf_i).ok()?;

    self.current_answer += 1;

    Some(answer)
  }
}
