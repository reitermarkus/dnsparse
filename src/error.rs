use core::fmt;

/// A DNS parsing error.
#[derive(Debug)]
pub enum Error {
  /// Message ended unexpectedly.
  MessageTooShort,
  /// Message exceeds maximum length.
  MessageTooLong,
  /// Wrong label pointer.
  Pointer,
  /// Name exceeded maximum length.
  NameTooLong,
}

impl fmt::Display for Error {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    f.write_str(match self {
      Error::MessageTooShort => "message too short",
      Error::MessageTooLong => "message too long",
      Error::Pointer => "invalid pointer",
      Error::NameTooLong => "name too long",
    })
  }
}

#[cfg(feature = "std")]
impl std::error::Error for Error {}
