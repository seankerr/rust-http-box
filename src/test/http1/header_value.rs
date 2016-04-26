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
fn header_value_eof() {
    let mut h = H{data: Vec::new()};
    let mut p = Parser::new(StreamType::Response);

    assert!(match p.parse(&mut h, b"HTTP/1.1 200 OK\r\nContent-Length: value") {
        Ok(Success::Eof(_)) => true,
        _ => false
    });

    assert_eq!(h.data, b"value");
    assert_eq!(p.get_state(), State::HeaderValue);
}

#[test]
fn header_value_complete() {
    let mut h = H{data: Vec::new()};
    let mut p = Parser::new(StreamType::Response);

    assert!(match p.parse(&mut h, b"HTTP/1.1 200 OK\r\nContent-Length: value\r") {
        Ok(Success::Eof(_)) => true,
        _ => false
    });

    assert_eq!(h.data, b"value");
    assert_eq!(p.get_state(), State::Newline2);
}

#[test]
fn header_value_multiline() {
    let mut h = H{data: Vec::new()};
    let mut p = Parser::new(StreamType::Response);

    assert!(match p.parse(&mut h, b"HTTP/1.1 200 OK\r\nContent-Length: value1\r\n value2\r") {
        Ok(Success::Eof(_)) => true,
        _ => false
    });

    assert_eq!(h.data, b"value1 value2");
    assert_eq!(p.get_state(), State::Newline2);
}

#[test]
fn header_value_white_space() {
    let mut h = H{data: Vec::new()};
    let mut p = Parser::new(StreamType::Response);

    assert!(match p.parse(&mut h, b"HTTP/1.1 200 OK\r\nContent-Length: \t \t \t \t value\r") {
        Ok(Success::Eof(_)) => true,
        _ => false
    });

    assert_eq!(h.data, b"value");
    assert_eq!(p.get_state(), State::Newline2);
}

#[test]
fn header_value_to_body() {
    let mut h = H{data: Vec::new()};
    let mut p = Parser::new(StreamType::Response);

    assert!(match p.parse(&mut h, b"HTTP/1.1 200 OK\r\nContent-Length: value\r\n\r\n") {
        Ok(Success::Eof(_)) => true,
        _ => false
    });

    assert_eq!(h.data, b"value");
    assert_eq!(p.get_state(), State::Body);
}
