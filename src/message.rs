use core::ops::Deref;
use core::mem::{size_of};
use core::fmt;

use crate::{Error, Answer, Answers, Header, Question, Questions, QueryKind, QueryClass, Name};

const HEADER_SIZE: usize = size_of::<Header>();
const MAX_MESSAGE_SIZE: usize = 512 - HEADER_SIZE;

/// Helper type for constructing a buffer with the maximum UDP message size.
pub type MessageBuffer = [u8; HEADER_SIZE + MAX_MESSAGE_SIZE];

/// A DNS message.
#[repr(C)]
pub struct Message<'a> {
  buf: &'a mut [u8],
  len: usize,
}

/// Builder for [`Message`](struct.Message.html).
#[derive(Debug)]
pub struct MessageBuilder<'a> {
  buf: &'a mut [u8],
  len: usize,
}

impl<'a> MessageBuilder<'a> {
  pub fn header(mut self, header: Header) -> Self {
    *self.header_mut() = header;

    self.len = HEADER_SIZE;

    self
  }

  fn header_mut(&mut self) -> &mut Header {
    unsafe { &mut *(self.buf[..HEADER_SIZE].as_mut_ptr() as *mut _ as *mut Header) }
  }

  pub fn build(self) -> Message<'a> {
    let Self { buf, len } = self;
    Message { buf, len }
  }
}

impl<'a> Message<'a> {
  pub const BUFFER: [u8; 512] = [0; 512];

  pub fn builder(buf: &'a mut [u8]) -> MessageBuilder<'a> {
    for b in &mut buf[..HEADER_SIZE] {
      *b = 0;
    }

    MessageBuilder { buf, len: HEADER_SIZE }
  }

  pub fn parse(buffer: &'a mut [u8]) -> Result<Message<'a>, Error> {
    if buffer.len() < HEADER_SIZE {
      return Err(Error::MessageTooShort)
    } else if buffer.len() > HEADER_SIZE + MAX_MESSAGE_SIZE {
      return Err(Error::MessageTooLong)
    }

    let len = buffer.len();
    let mut frame = Self { buf: buffer, len };

    let mut i = HEADER_SIZE;

    for _ in 0..frame.header().question_count() {
      Question::read(frame.buf, &mut i)?;
    }

    for _ in 0..frame.header().answer_count() {
      Answer::read(frame.buf, &mut i)?;
    }

    frame.len = i;

    Ok(frame)
  }
}

impl fmt::Debug for Message<'_> {
  fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
    fmt.debug_struct("Message")
      .field("header", &self.header())
      .field("body", &format_args!("{:?}", &self.buf[HEADER_SIZE..]))
      .field("len", &self.len)
      .finish()
  }
}

impl Deref for Message<'_> {
  type Target = [u8];

  fn deref(&self) -> &Self::Target {
    self.as_bytes()
  }
}

impl Message<'_> {
  pub fn header(&self) -> &Header {
    unsafe { &*(self.buf[..HEADER_SIZE].as_ptr() as *const _ as *const Header) }
  }

  pub fn header_mut(&mut self) -> &mut Header {
    unsafe { &mut *(self.buf[..HEADER_SIZE].as_mut_ptr() as *mut _ as *mut Header) }
  }

  pub fn add_question(&mut self, question: &Question<'_>) {
    let mut i = self.questions_end();
    self.add_name(&mut i, &question.name);
    self.add_kind(&mut i, &question.kind);
    self.add_class(&mut i, &question.class);

    let header = self.header_mut();
    unsafe { header.set_question_count(header.question_count() + 1) };
  }

  pub fn add_answer(&mut self, answer: &Answer<'_>) {
    let mut i = self.answers_end();
    self.add_name(&mut i, &answer.name);
    self.add_kind(&mut i, &answer.kind);
    self.add_class(&mut i, &answer.class);
    self.add_ttl(&mut i, answer.ttl);
    self.add_rdata(&mut i, answer.rdata);

    let header = self.header_mut();
    unsafe { header.set_answer_count(header.answer_count() + 1) };
  }

  fn add_pointer(&mut self, i: &mut usize, name: &Name<'_>) -> bool {
    for question in self.questions() {
      if let Some(pointer) = question.name().create_pointer(name) {
        self.insert(i, &pointer);
        return true
      }
    }

    false
  }

  fn add_name(&mut self, i: &mut usize, name: &Name<'_>) {
    if self.add_pointer(i, name) {
      return;
    }

    let mut name_rest: Option<Name<'_>> = None;
    loop {
      let (label, name) = if let Some(name) = name_rest {
        name.split()
      } else {
        name.split()
      };

      self.insert(i, &[label.len() as u8]);
      self.insert(i, label.as_bytes());

      match name {
        Some(rest) => {
          if self.add_pointer(i, &rest) {
            return;
          }

          name_rest = Some(rest);
        },
        None => break,
      }
    }

    self.insert(i, &[0]);
  }

  fn add_kind(&mut self, i: &mut usize, kind: &QueryKind) {
    self.insert(i, &kind.to_be_bytes());
  }

  fn add_class(&mut self, i: &mut usize, class: &QueryClass) {
    self.insert(i, &class.to_be_bytes());
  }

  fn add_ttl(&mut self, i: &mut usize, ttl: u32) {
    self.insert(i, &ttl.to_be_bytes());
  }

  fn add_rdata(&mut self, i: &mut usize, data: &[u8]) {
    self.insert(i, &(data.len() as u16).to_be_bytes());
    self.insert(i, data);
  }

  fn insert(&mut self, i: &mut usize, bytes: &[u8]) {
    let len = bytes.len();

    for j in (*i..self.len).rev() {
      self.buf[j + len] = self.buf[j];
    }

    self.buf[*i..(*i + len)].copy_from_slice(bytes);

    self.len += len;

    *i += len;
  }

  pub fn as_bytes(&self) -> &[u8] {
    &self.buf[..self.len]
  }

  pub fn questions(&self) -> Questions<'_> {
    Questions {
      question_count: self.header().question_count() as usize,
      current_question: 0,
      buf: self.as_bytes(),
      buf_i: HEADER_SIZE,
    }
  }

  fn questions_end(&self) -> usize {
    let buf = &self.as_bytes();
    let mut i = HEADER_SIZE;

    for _ in 0..self.header().question_count() {
      assert!(Question::read(buf, &mut i).is_ok());
    }

    i
  }

  pub fn answers(&self) -> Answers<'_> {
    Answers {
      answer_count: self.header().answer_count() as usize,
      current_answer: 0,
      buf: self.as_bytes(),
      buf_i: self.questions_end(),
    }
  }

  fn answers_end(&self) -> usize {
    let buf = &self.as_bytes();
    let mut i = self.questions_end();

    for _ in 0..self.header().answer_count() {
      assert!(Answer::read(buf, &mut i).is_ok());
    }

    i
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_name_split() {
    let mut question = [71, 208, 1, 0, 0, 1, 0, 0, 0, 0, 0, 0, 7, 99, 97, 112, 116, 105, 118, 101, 5, 97, 112, 112, 108, 101, 3, 99, 111, 109, 0, 0, 1, 0, 1];

    let message = Message::parse(&mut question).unwrap();

    let question = message.questions().next().unwrap();
    let name = question.name();

    let (label, name) = name.split();
    assert_eq!(label.as_str(), "captive");

    let (label, name) = name.unwrap().split();
    assert_eq!(label.as_str(), "apple");

    let (label, name) = name.unwrap().split();
    assert_eq!(label.as_str(), "com");

    assert!(name.is_none());
  }
}
