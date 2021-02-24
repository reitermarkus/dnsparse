use core::fmt;

/// A DNS header.
#[derive(Clone)]
#[repr(C)]
pub struct Header {
  id: [u8; 2],
  flags: [u8; 2],
  question_count: [u8; 2],
  answer_count: [u8; 2],
  name_server_count: [u8; 2],
  additional_records_count: [u8; 2],
}

/// The kind of a DNS header.
#[derive(Debug, PartialEq)]
pub enum HeaderKind {
  Query,
  Response,
}

/// A DNS opcode.
#[derive(Debug, PartialEq)]
pub enum OpCode {
  Query,
  InverseQuery,
  Status,
  Notify,
  Update,
  Reserved(u8)
}

/// A DNS response code.
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum ResponseCode {
  NoError,
  FormatError,
  ServerFailure,
  NonExistentDomain,
  NotImplemented,
  Refused,
  ExistentDomain,
  ExistentRrSet,
  NonExistentRrSet,
  NotAuthoritative,
  NotZone,
  BadOptVersionOrBadSignature,
  BadKey,
  BadTime,
  BadMode,
  BadName,
  BadAlg,
  Reserved(u16),
}

impl From<ResponseCode> for u16 {
  fn from(r: ResponseCode) -> Self {
    match r {
      ResponseCode::NoError => 0,
      ResponseCode::FormatError => 1,
      ResponseCode::ServerFailure => 2,
      ResponseCode::NonExistentDomain => 3,
      ResponseCode::NotImplemented => 4,
      ResponseCode::Refused => 5,
      ResponseCode::ExistentDomain => 6,
      ResponseCode::ExistentRrSet => 7,
      ResponseCode::NonExistentRrSet => 8,
      ResponseCode::NotAuthoritative => 9,
      ResponseCode::NotZone => 10,
      ResponseCode::BadOptVersionOrBadSignature => 16,
      ResponseCode::BadKey => 17,
      ResponseCode::BadTime => 18,
      ResponseCode::BadMode => 19,
      ResponseCode::BadName => 20,
      ResponseCode::BadAlg => 21,
      ResponseCode::Reserved(n) => n
    }
  }
}

impl From<u16> for ResponseCode {
  fn from(n: u16) -> Self {
    match n {
      0 => ResponseCode::NoError,
      1 => ResponseCode::FormatError,
      2 => ResponseCode::ServerFailure,
      3 => ResponseCode::NonExistentDomain,
      4 => ResponseCode::NotImplemented,
      5 => ResponseCode::Refused,
      6 => ResponseCode::ExistentDomain,
      7 => ResponseCode::ExistentRrSet,
      8 => ResponseCode::NonExistentRrSet,
      9 => ResponseCode::NotAuthoritative,
      10 => ResponseCode::NotZone,
      16 => ResponseCode::BadOptVersionOrBadSignature,
      17 => ResponseCode::BadKey,
      18 => ResponseCode::BadTime,
      19 => ResponseCode::BadMode,
      20 => ResponseCode::BadName,
      21 => ResponseCode::BadAlg,
      n => ResponseCode::Reserved(n),
    }
  }
}

impl fmt::Debug for Header {
  fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
    fmt.debug_struct("Header")
      .field("id", &self.id())
      .field("kind", &self.kind())
      .field("opcode", &self.opcode())
      .field("authoritative_answer", &self.authoritative_answer())
      .field("truncated", &self.truncated())
      .field("recursion_desired", &self.recursion_desired())
      .field("recursion_available", &self.recursion_available())
      .field("response_code", &self.response_code())
      .field("question_count", &self.question_count())
      .field("answer_count", &self.answer_count())
      .field("name_server_count", &self.name_server_count())
      .field("additional_records_count", &self.additional_records_count())
      .finish()
  }
}

impl Header {
  #[inline]
  pub fn id(&self) -> u16 {
    u16::from_be_bytes(self.id)
  }

  #[inline]
  pub fn set_id(&mut self, id: u16) {
    self.id = id.to_be_bytes()
  }

  #[inline]
  pub fn kind(&self) -> HeaderKind {
    if (self.flags[0] & 0b10000000) == 0 {
      HeaderKind::Query
    } else {
      HeaderKind::Response
    }
  }

  #[inline]
  pub fn set_kind(&mut self, kind: HeaderKind) {
    match kind {
      HeaderKind::Query    => self.flags[0] &= 0b01111111,
      HeaderKind::Response => self.flags[0] |= 0b10000000,
    }
  }

  #[inline]
  pub fn opcode(&self) -> OpCode {
    match (self.flags[0] & 0b01111000) >> 3 {
      0 => OpCode::Query,
      1 => OpCode::InverseQuery,
      2 => OpCode::Status,
      4 => OpCode::Notify,
      5 => OpCode::Update,
      n => OpCode::Reserved(n),
    }
  }

  #[inline]
  pub fn set_opcode(&mut self, opcode: OpCode) {
    self.flags[0] = (self.flags[0] & 0b10000111) | (match opcode {
      OpCode::Query => 0,
      OpCode::InverseQuery => 1,
      OpCode::Status => 2,
      OpCode::Notify => 4,
      OpCode::Update => 5,
      OpCode::Reserved(n) => n,
    } << 3);
  }

  #[inline]
  pub fn authoritative_answer(&self) -> bool {
    (self.flags[0] & 0b00000100) != 0
  }

  #[inline]
  pub fn truncated(&self) -> bool {
    (self.flags[0] & 0b00000010) != 0
  }

  #[inline]
  pub fn recursion_desired(&self) -> bool {
    (self.flags[0] & 0b00000001) != 0
  }

  #[inline]
  pub fn set_recursion_desired(&mut self, recursion_desired: bool) {
    if recursion_desired {
      self.flags[0] |= 0b00000001;
    } else {
      self.flags[0] &= 0b11111110;
    }
  }

  #[inline]
  pub fn recursion_available(&self) -> bool {
    (self.flags[1] & 0b10000000) != 0
  }

  #[inline]
  pub fn set_recursion_available(&mut self, recursion_available: bool) {
    if recursion_available {
      self.flags[1] |= 0b10000000;
    } else {
      self.flags[1] &= 0b01111111;
    }
  }

  #[inline]
  pub fn response_code(&self) -> ResponseCode {
    u16::from(self.flags[1] & 0b00001111).into()
  }

  #[inline]
  pub fn set_response_code(&mut self, response_code: ResponseCode) {
    self.flags[1] = (self.flags[1] & 0b11110000) | u16::from(response_code) as u8;
  }

  #[inline]
  pub fn question_count(&self) -> u16 {
    u16::from_be_bytes(self.question_count)
  }

  /// # Safety
  ///
  /// Setting the question count manually is unsafe. It is the caller's responsibility to
  /// ensure that the corresponding message contains the right amount of questions.
  #[inline]
  pub unsafe fn set_question_count(&mut self, question_count: u16) {
    self.question_count = question_count.to_be_bytes();
  }

  #[inline]
  pub fn answer_count(&self) -> u16 {
    u16::from_be_bytes(self.answer_count)
  }

  /// # Safety
  ///
  /// Setting the answer count manually is unsafe. It is the caller's responsibility to
  /// ensure that the corresponding message contains the right amount of answers.
  #[inline]
  pub unsafe fn set_answer_count(&mut self, answer_count: u16) {
    self.answer_count = answer_count.to_be_bytes();
  }

  #[inline]
  pub fn name_server_count(&self) -> u16 {
    u16::from_be_bytes(self.name_server_count)
  }

  /// # Safety
  ///
  /// Setting the name server count manually is unsafe. It is the caller's responsibility to
  /// ensure that the corresponding message contains the right amount of name servers.
  #[inline]
  pub unsafe fn set_name_server_count(&mut self, name_server_count: u16) {
    self.name_server_count = name_server_count.to_be_bytes();
  }

  #[inline]
  pub fn additional_records_count(&self) -> u16 {
    u16::from_be_bytes(self.additional_records_count)
  }

  /// # Safety
  ///
  /// Setting the additional records count manually is unsafe. It is the caller's responsibility to
  /// ensure that the corresponding message contains the right amount of additional records.
  #[inline]
  pub unsafe fn set_additional_records_count(&mut self, additional_records_count: u16) {
    self.additional_records_count = additional_records_count.to_be_bytes();
  }

  #[inline]
  pub fn builder() -> HeaderBuilder {
    HeaderBuilder::new()
  }
}

/// Builder for [`Header`](struct.Header.html).
#[derive(Debug)]
pub struct HeaderBuilder(Header);

impl Default for HeaderBuilder {
  fn default() -> Self {
    Self(Header {
      id: [0, 0],
      flags: [0, 0],
      question_count: [0, 0],
      answer_count: [0, 0],
      name_server_count: [0, 0],
      additional_records_count: [0, 0],
    })
  }
}

impl HeaderBuilder {
  #[inline]
  pub fn new() -> Self {
    Self::default()
  }

  pub fn id(mut self, id: u16) -> Self {
    self.0.set_id(id);
    self
  }

  pub fn kind(mut self, kind: HeaderKind) -> Self {
    self.0.set_kind(kind);
    self
  }

  pub fn recursion_desired(mut self, recursion_desired: bool) -> Self {
    self.0.set_recursion_desired(recursion_desired);
    self
  }

  pub fn recursion_available(mut self, recursion_available: bool) -> Self {
    self.0.set_recursion_available(recursion_available);
    self
  }

  pub fn response_code(mut self, response_code: ResponseCode) -> Self {
    self.0.set_response_code(response_code);
    self
  }

  pub fn build(self) -> Header {
    self.0
  }
}
