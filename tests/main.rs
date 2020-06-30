use dnsparse::*;

const CAPTIVE_APPLE_COM_QUERY: [u8; 48] = [
  30, 252,                                     // ID
  1, 0,                                        // Kind
  0, 1,                                        // Question Count
  0, 0,                                        // Answer Count
  0, 0,                                        // Name Server Count
  0, 0,                                        // Additional Records Count
  7, b'c', b'a', b'p', b't', b'i', b'v', b'e', // Label "captive"
  5, b'a', b'p', b'p', b'l', b'e',             // Label "apple"
  3, b'c', b'o', b'm',                         // Label "com"
  0,                                           // Label End
  0, 1,                                        // Question Class
  0, 1,                                        // Question Kind
  0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,       // Padding for testing `Message::len`.
];

macro_rules! parse {
  ($req:ident) => {
    let mut buf = CAPTIVE_APPLE_COM_QUERY;
    let $req = Message::parse(&mut buf).expect("parsing failed");
  }
}

#[test]
fn test_parse() {
  let mut buf = CAPTIVE_APPLE_COM_QUERY;
  let request = Message::parse(&mut buf).expect("parsing failed");
  assert_eq!(request.len(), 35);
}

#[test]
fn test_header_id() {
  parse!(request);
  assert_eq!(request.header().id(), 7932);
}

#[test]
fn test_header_kind() {
  parse!(request);
  assert_eq!(request.header().kind(), HeaderKind::Query);
}

#[test]
fn test_header_opcode() {
  parse!(request);
  assert_eq!(request.header().opcode(), OpCode::Query);
}


#[test]
fn test_header_authoritative_answer() {
  parse!(request);
  assert_eq!(request.header().authoritative_answer(), false);
}

#[test]
fn test_header_truncated() {
  parse!(request);
  assert_eq!(request.header().truncated(), false);
}

#[test]
fn test_header_recursion_desired() {
  parse!(request);
  assert_eq!(request.header().recursion_desired(), true);
}

#[test]
fn test_header_recursion_available() {
  parse!(request);
  assert_eq!(request.header().recursion_available(), false);
}

#[test]
fn test_header_response_code() {
  parse!(request);
  assert_eq!(request.header().response_code(), ResponseCode::NoError);
}

#[test]
fn test_header_question_count() {
  parse!(request);
  assert_eq!(request.header().question_count(), 1);
}

#[test]
fn test_header_answer_count() {
  parse!(request);
  assert_eq!(request.header().answer_count(), 0);
}

#[test]
fn test_header_name_server_count() {
  parse!(request);
  assert_eq!(request.header().name_server_count(), 0);
}

#[test]
fn test_header_additional_records_count() {
  parse!(request);
  assert_eq!(request.header().additional_records_count(), 0);
}

#[test]
fn test_questions() {
  parse!(request);

  let mut questions = request.questions();

  let question = questions.next().expect("`Questions` iterator is empty");

  assert_eq!(question.name(), "captive.apple.com");
  assert_eq!(question.name(), "CAPTIVE.APPLE.COM");
  assert_ne!(question.name(), ".captive.apple.com");
  assert_ne!(question.name(), "captive.apple.com.");
  assert_ne!(question.name(), "captive-apple-com");
  assert_eq!(*question.class(), QueryClass::IN);
  assert_eq!(*question.kind(), QueryKind::A);

  assert_eq!(question.name().to_string(), "captive.apple.com".to_string());

  assert!(questions.next().is_none());
}

#[test]
fn test_as_bytes() {
  parse!(request);

  let bytes = request.as_bytes();
  assert_eq!(bytes.len(), request.len());
  assert_eq!(&CAPTIVE_APPLE_COM_QUERY[..bytes.len()], bytes);
}
