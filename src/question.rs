use core::fmt;

use crate::{Error, Name, QueryKind, QueryClass};

/// A DNS question.
#[repr(C)]
pub struct Question<'a> {
  pub(crate) name: Name<'a>,
  pub(crate) kind: QueryKind,
  pub(crate) class: QueryClass,
}

impl fmt::Debug for Question<'_> {
  fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
    fmt.debug_struct("Question")
      .field("name", &self.name())
      .field("kind", &self.kind())
      .field("class", &self.class())
      .finish()
  }
}

impl<'a> Question<'a> {
  pub(crate) fn read(buf: &'a [u8], i: &'_ mut usize) -> Result<Self, Error> {
    let mut j = *i;
    let question = Self {
      name:  Name::read(buf, &mut j)?,
      kind:  QueryKind::read(buf, &mut j)?,
      class: QueryClass::read(buf, &mut j)?,
    };
    *i = j;

    Ok(question)
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

    let question = Question::read(self.buf, &mut i).ok()?;

    self.current_question += 1;
    self.buf_i = i;

    Some(question)
  }
}
