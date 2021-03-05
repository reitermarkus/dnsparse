#![cfg_attr(not(any(test, feature = "std")), no_std)]
#![deny(missing_debug_implementations, rust_2018_idioms)]

//! A `no_std` library for parsing and generating DNS queries and responses.
//!
//! Implemented according to [RFC 1035](https://tools.ietf.org/rfc/rfc1035).

mod error;
pub use error::Error;

mod message;
pub use message::{Message, MessageBuilder, MessageBuffer};

mod header;
pub use header::{Header, HeaderKind, ResponseCode, OpCode};

mod query_kind;
pub use query_kind::QueryKind;

mod query_class;
pub use query_class::QueryClass;

mod question;
pub use question::{Question, Questions};

mod name;
pub use name::Name;

mod answer;
pub use answer::{Answer, Answers};
