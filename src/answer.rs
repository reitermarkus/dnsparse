use core::mem::size_of;

use crate::{Name, QueryKind, QueryClass};

/// A DNS answer.
#[derive(Debug)]
pub struct Answer<'a> {
  pub name: Name<'a>,
  pub kind: QueryKind,
  pub class: QueryClass,
  pub ttl: u32,
  pub rdata: &'a [u8],
}

fn read_ttl(buf: &[u8], i: &mut usize) -> bool {
  if *i + size_of::<u32>() <= buf.len() {
    *i += size_of::<u32>();
    true
  } else {
    false
  }
}

fn read_rdata(buf: &[u8], i: &mut usize) -> bool {
  if *i + size_of::<u16>() <= buf.len() {
    *i += size_of::<u16>();

    let rdata_len = u16::from_be_bytes([buf[*i - 2], buf[*i - 1]]) as usize;
    if *i + rdata_len <= buf.len() {
      *i += rdata_len;
      return true;
    }
  }

  false
}

impl<'a> Answer<'a> {
  #[inline]
  pub(crate) fn read(buf: &[u8], i: &mut usize) -> bool {
    Name::read(buf, i) && QueryKind::read(buf, i) && QueryClass::read(buf, i) && read_ttl(buf, i) && read_rdata(buf, i)
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
    &self.rdata
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

    let mut i = self.buf_i;

    // `Answers` can only be created from a valid `Message`, so
    // reading an `Answer` should always succeed here.
    assert!(Name::read(&self.buf, &mut i));
    let kind_i = i;
    assert!(QueryKind::read(&self.buf, &mut i));
    let class_i = i;
    assert!(QueryClass::read(&self.buf, &mut i));
    let ttl_i = i;
    assert!(read_ttl(&self.buf, &mut i));
    let rdata_len_i = i;
    assert!(read_rdata(&self.buf, &mut i));
    let rdata_i = rdata_len_i + 2;

    let rdata_len = u16::from_be_bytes([self.buf[rdata_len_i], self.buf[rdata_len_i + 1]]) as usize;

    let answer = Answer {
      name: Name {
        buf: &self.buf,
        start: self.buf_i,
      },
      kind: QueryKind::from(u16::from_be_bytes([self.buf[kind_i], self.buf[kind_i + 1]])),
      class: QueryClass::from(u16::from_be_bytes([self.buf[class_i], self.buf[class_i + 1]])),
      ttl: u32::from_be_bytes([self.buf[ttl_i], self.buf[ttl_i + 1], self.buf[ttl_i + 2], self.buf[ttl_i + 3]]),
      rdata: &self.buf[rdata_i..(rdata_i + rdata_len)],
    };

    self.current_answer += 1;
    self.buf_i = i;

    Some(answer)
  }
}
