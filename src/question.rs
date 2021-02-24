use core::fmt;

use crate::{Name, QueryKind, QueryClass};

/// A DNS question.
#[repr(C)]
pub struct Question<'a> {
  pub(crate) name: Name<'a>,
  pub(crate) kind: QueryKind,
  pub(crate) class: QueryClass,
}

impl fmt::Debug for Question<'_> {
  fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
    fmt.debug_struct("Question")
      .field("name", &self.name())
      .field("kind", &self.kind())
      .field("class", &self.class())
      .finish()
  }
}

impl<'a> Question<'a> {
  #[inline]
  pub(crate) fn read(buf: &[u8], i: &mut usize) -> bool {
    Name::read(buf, i) && QueryKind::read(buf, i) && QueryClass::read(buf, i)
  }

  #[inline]
  pub fn name(&self) -> &Name<'a> {
    &self.name
  }

  #[inline]
  pub fn kind(&self) -> &QueryKind {
    &self.kind
  }

  #[inline]
  pub fn class(&self) -> &QueryClass {
    &self.class
  }
}

/// Iterator over [`Question`](struct.Question.html)s contained in a [`Message`](struct.Message.html).
#[derive(Debug)]
pub struct Questions<'a> {
  pub(crate) question_count: usize,
  pub(crate) current_question: usize,
  pub(crate) buf: &'a [u8],
  pub(crate) buf_i: usize,
}

impl<'a> Iterator for Questions<'a> {
  type Item = Question<'a>;

  fn next(&mut self) -> Option<Self::Item> {
    if self.current_question >= self.question_count {
      return None
    }

    let mut i = self.buf_i;

    // `Questions` can only be created from a valid `Message`, so
    // reading a `Question` should always succeed here.
    assert!(Question::read(&self.buf, &mut i));
    let question = Question {
      name: Name {
        buf: &self.buf,
        start: self.buf_i,
      },
      kind: QueryKind::from(u16::from_be_bytes([self.buf[i - 4], self.buf[i - 3]])),
      class: QueryClass::from(u16::from_be_bytes([self.buf[i - 2], self.buf[i - 1]])),
    };

    self.current_question += 1;
    self.buf_i = i;

    Some(question)
  }
}
