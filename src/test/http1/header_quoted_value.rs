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
    data: Vec<u8>
}

impl HttpHandler for H {
    fn on_header_value(&mut self, data: &[u8]) -> bool {
        println!("on_header_value: {:?}", str::from_utf8(data).unwrap());
        self.data.extend_from_slice(data);
        true
    }
}

impl ParamHandler for H {}

#[test]
fn header_quoted_value_escape() {
    let mut h = H{data: Vec::new()};
    let mut p = Parser::new(StreamType::Response);

    assert!(match p.parse(&mut h, b"HTTP/1.1 200 OK\r\nCustom-Header: \"multiple \\\"word\\\" value\"\r") {
        Ok(Success::Eof(_)) => true,
        _ => false
    });

    assert_eq!(h.data, b"multiple \"word\" value");
    assert_eq!(p.get_state(), State::Newline2);
}

#[test]
fn header_quoted_value_no_escape() {
    let mut h = H{data: Vec::new()};
    let mut p = Parser::new(StreamType::Response);

    assert!(match p.parse(&mut h, b"HTTP/1.1 200 OK\r\nCustom-Header: \"multiple word value\"\r") {
        Ok(Success::Eof(_)) => true,
        _ => false
    });

    assert_eq!(h.data, b"multiple word value");
    assert_eq!(p.get_state(), State::Newline2);
}

#[test]
fn header_quoted_value_starting_escape() {
    let mut h = H{data: Vec::new()};
    let mut p = Parser::new(StreamType::Response);

    assert!(match p.parse(&mut h, b"HTTP/1.1 200 OK\r\nCustom-Header: \"\\\"word\\\" value\"\r") {
        Ok(Success::Eof(_)) => true,
        _ => false
    });

    assert_eq!(h.data, b"\"word\" value");
    assert_eq!(p.get_state(), State::Newline2);
}

#[test]
fn header_quoted_value_ending_escape() {
    let mut h = H{data: Vec::new()};
    let mut p = Parser::new(StreamType::Response);

    assert!(match p.parse(&mut h, b"HTTP/1.1 200 OK\r\nCustom-Header: \"word \\\"value\\\"\"\r") {
        Ok(Success::Eof(_)) => true,
        _ => false
    });

    assert_eq!(h.data, b"word \"value\"");
    assert_eq!(p.get_state(), State::Newline2);
}

#[test]
fn header_quoted_value_multiline() {
    let mut h = H{data: Vec::new()};
    let mut p = Parser::new(StreamType::Response);

    assert!(match p.parse(&mut h, b"HTTP/1.1 200 OK\r\nContent-Length: \"value1\"\r\n \"value2\"\r\n \"value3\"\r\n \"value4\"\r") {
        Ok(Success::Eof(_)) => true,
        _ => false
    });

    assert_eq!(h.data, b"value1 value2 value3 value4");
    assert_eq!(p.get_state(), State::Newline2);
}

#[test]
fn header_quoted_value_white_space() {
    let mut h = H{data: Vec::new()};
    let mut p = Parser::new(StreamType::Response);

    assert!(match p.parse(&mut h, b"HTTP/1.1 200 OK\r\nContent-Length: \t \t \t \t \"value\"\r") {
        Ok(Success::Eof(_)) => true,
        _ => false
    });

    assert_eq!(h.data, b"value");
    assert_eq!(p.get_state(), State::Newline2);
}

#[test]
fn header_quoted_value_incomplete() {
    let mut h = H{data: Vec::new()};
    let mut p = Parser::new(StreamType::Response);

    assert!(match p.parse(&mut h, b"HTTP/1.1 200 OK\r\nCustom-Header: \"multiple word\r") {
        Err(ParserError::HeaderValue(_,_)) => true,
        _ => false
    });

    assert_eq!(p.get_state(), State::Dead);
}

#[test]
fn header_quoted_value_invalid_byte() {
    let mut h = H{data: Vec::new()};
    let mut p = Parser::new(StreamType::Response);

    assert!(match p.parse(&mut h, b"HTTP/1.1 200 OK\r\nCustom-Header: \"\0value\"\r") {
        Err(ParserError::HeaderValue(_,_)) => true,
        _ => false
    });

    assert_eq!(p.get_state(), State::Dead);
}
