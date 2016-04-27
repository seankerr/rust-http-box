// +-----------------------------------------------------------------------------------------------+
// | Copyright 2016 Sean Kerr                                                                      |
// |                                                                                               |
// | Licensed under the Apache License, Version 2.0 (the "License");                               |
// | you may not use this file except in compliance with the License.                              |
// | You may obtain a copy of the License at                                                       |
// |                                                                                               |
// |  http://www.apache.org/licenses/LICENSE-2.0                                                   |
// |                                                                                               |
// | Unless required by applicable law or agreed to in writing, software                           |
// | distributed under the License is distributed on an "AS IS" BASIS,                             |
// | WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.                      |
// | See the License for the specific language governing permissions and                           |
// | limitations under the License.                                                                |
// +-----------------------------------------------------------------------------------------------+
// | Author: Sean Kerr <sean@code-box.org>                                                         |
// +-----------------------------------------------------------------------------------------------+

use Success;
use http1::*;
use url::*;
use std::str;

struct H {
    content_length: ContentLength,
    field:          Vec<u8>,
    finished:       bool,
    value:          Vec<u8>
}

impl HttpHandler for H {
    fn get_content_length(&mut self) -> ContentLength {
        match self.content_length {
            ContentLength::None => {
                println!("get_content_length: none");
            },
            ContentLength::Specified(length) => {
                println!("get_content_length: {}", length);
            }
        }

        self.content_length
    }

    fn get_content_type(&mut self) -> ContentType {
        println!("get_content_type: urlencoded");
        ContentType::UrlEncoded
    }

    fn get_transfer_encoding(&mut self) -> TransferEncoding {
        println!("get_transfer_encoding: none");
        TransferEncoding::None
    }

    fn on_finished(&mut self) {
        self.finished = true;
    }
}

impl ParamHandler for H {
    fn on_param_field(&mut self, data: &[u8]) -> bool {
        println!("on_param_field: {:?}", str::from_utf8(data).unwrap());
        self.field.extend_from_slice(data);
        true
    }

    fn on_param_value(&mut self, data: &[u8]) -> bool {
        println!("on_param_value: {:?}", str::from_utf8(data).unwrap());
        self.value.extend_from_slice(data);
        true
    }
}

#[test]
fn urlencoded_field_basic() {
    let mut h = H{content_length: ContentLength::None, field: Vec::new(),
                  finished: false, value: Vec::new()};
    let mut p = Parser::new(StreamType::Response);

    assert!(match p.parse(&mut h, b"HTTP/1.1 200 OK\r\n\r\nParam") {
        Ok(Success::Eof(_)) => {
            assert_eq!(h.field, b"Param");
            assert_eq!(p.get_state(), State::UrlEncodedField);
            true
        },
        _ => false
    });
}

#[test]
fn urlencoded_field_basic_ending_equal() {
    let mut h = H{content_length: ContentLength::None, field: Vec::new(),
                  finished: false, value: Vec::new()};
    let mut p = Parser::new(StreamType::Response);

    assert!(match p.parse(&mut h, b"HTTP/1.1 200 OK\r\n\r\nParam=") {
        Ok(Success::Eof(_)) => {
            assert_eq!(h.field, b"Param");
            assert_eq!(p.get_state(), State::UrlEncodedValue);
            true
        },
        _ => false
    });
}

#[test]
fn urlencoded_field_basic_ending_percent() {
    let mut h = H{content_length: ContentLength::None, field: Vec::new(),
                  finished: false, value: Vec::new()};
    let mut p = Parser::new(StreamType::Response);

    assert!(match p.parse(&mut h, b"HTTP/1.1 200 OK\r\n\r\nParam%") {
        Ok(Success::Eof(24)) => {
            assert_eq!(h.field, b"Param");
            assert_eq!(p.get_state(), State::UrlEncodedFieldHex);
            true
        },
        _ => false
    });
}

#[test]
fn urlencoded_field_basic_ending_plus() {
    let mut h = H{content_length: ContentLength::None, field: Vec::new(),
                  finished: false, value: Vec::new()};
    let mut p = Parser::new(StreamType::Response);

    assert!(match p.parse(&mut h, b"HTTP/1.1 200 OK\r\n\r\nParam+") {
        Ok(Success::Eof(25)) => {
            assert_eq!(h.field, b"Param ");
            assert_eq!(p.get_state(), State::UrlEncodedField);
            true
        },
        _ => false
    });
}

#[test]
fn urlencoded_field_basic_ending_ampersand() {
    let mut h = H{content_length: ContentLength::None, field: Vec::new(),
                  finished: false, value: Vec::new()};
    let mut p = Parser::new(StreamType::Response);

    assert!(match p.parse(&mut h, b"HTTP/1.1 200 OK\r\n\r\nParam&") {
        Ok(Success::Eof(25)) => {
            assert_eq!(h.field, b"Param");
            assert_eq!(h.value, b"");
            assert_eq!(p.get_state(), State::UrlEncodedField);
            true
        },
        _ => false
    });
}

#[test]
fn urlencoded_field_hex() {
    let mut h = H{content_length: ContentLength::None, field: Vec::new(),
                  finished: false, value: Vec::new()};
    let mut p = Parser::new(StreamType::Response);

    assert!(match p.parse(&mut h, b"HTTP/1.1 200 OK\r\n\r\nParam%201") {
        Ok(Success::Eof(28)) => {
            assert_eq!(h.field, b"Param 1");
            assert_eq!(p.get_state(), State::UrlEncodedField);
            true
        },
        _ => false
    });
}

#[test]
fn urlencoded_field_hex_illegal() {
    let mut h = H{content_length: ContentLength::None, field: Vec::new(),
                  finished: false, value: Vec::new()};
    let mut p = Parser::new(StreamType::Response);

    assert!(match p.parse(&mut h, b"HTTP/1.1 200 OK\r\n\r\nParam%xx") {
        Err(ParserError::UrlEncodedField(_,_)) => {
            assert_eq!(p.get_state(), State::Dead);
            true
        },
        _ => false
    });
}

#[test]
fn urlencoded_value_basic() {
    let mut h = H{content_length: ContentLength::None, field: Vec::new(),
                  finished: false, value: Vec::new()};
    let mut p = Parser::new(StreamType::Response);

    assert!(match p.parse(&mut h, b"HTTP/1.1 200 OK\r\n\r\nParam=Value") {
        Ok(Success::Eof(_)) => {
            assert_eq!(h.field, b"Param");
            assert_eq!(h.value, b"Value");
            assert_eq!(p.get_state(), State::UrlEncodedValue);
            true
        },
        _ => false
    });
}

#[test]
fn urlencoded_value_basic_ending_percent() {
    let mut h = H{content_length: ContentLength::None, field: Vec::new(),
                  finished: false, value: Vec::new()};
    let mut p = Parser::new(StreamType::Response);

    assert!(match p.parse(&mut h, b"HTTP/1.1 200 OK\r\n\r\nParam=Value%") {
        Ok(Success::Eof(30)) => {
            assert_eq!(h.field, b"Param");
            assert_eq!(h.value, b"Value");
            assert_eq!(p.get_state(), State::UrlEncodedValueHex);
            true
        },
        _ => false
    });
}

#[test]
fn urlencoded_value_basic_ending_plus() {
    let mut h = H{content_length: ContentLength::None, field: Vec::new(),
                  finished: false, value: Vec::new()};
    let mut p = Parser::new(StreamType::Response);

    assert!(match p.parse(&mut h, b"HTTP/1.1 200 OK\r\n\r\nParam=Value+") {
        Ok(Success::Eof(31)) => {
            assert_eq!(h.field, b"Param");
            assert_eq!(h.value, b"Value ");
            assert_eq!(p.get_state(), State::UrlEncodedValue);
            true
        },
        _ => false
    });
}

#[test]
fn urlencoded_value_basic_ending_ampersand() {
    let mut h = H{content_length: ContentLength::None, field: Vec::new(),
                  finished: false, value: Vec::new()};
    let mut p = Parser::new(StreamType::Response);

    assert!(match p.parse(&mut h, b"HTTP/1.1 200 OK\r\n\r\nParam=Value&") {
        Ok(Success::Eof(31)) => {
            assert_eq!(h.field, b"Param");
            assert_eq!(h.value, b"Value");
            assert_eq!(p.get_state(), State::UrlEncodedField);
            true
        },
        _ => false
    });
}

#[test]
fn urlencoded_value_hex() {
    let mut h = H{content_length: ContentLength::None, field: Vec::new(),
                  finished: false, value: Vec::new()};
    let mut p = Parser::new(StreamType::Response);

    assert!(match p.parse(&mut h, b"HTTP/1.1 200 OK\r\n\r\nParam=Value%201") {
        Ok(Success::Eof(34)) => {
            assert_eq!(h.field, b"Param");
            assert_eq!(h.value, b"Value 1");
            assert_eq!(p.get_state(), State::UrlEncodedValue);
            true
        },
        _ => false
    });
}

#[test]
fn urlencoded_value_hex_illegal() {
    let mut h = H{content_length: ContentLength::None, field: Vec::new(),
                  finished: false, value: Vec::new()};
    let mut p = Parser::new(StreamType::Response);

    assert!(match p.parse(&mut h, b"HTTP/1.1 200 OK\r\n\r\nParam=Value%xx") {
        Err(ParserError::UrlEncodedValue(_,_)) => {
            assert_eq!(p.get_state(), State::Dead);
            true
        },
        _ => false
    });
}
