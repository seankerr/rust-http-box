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
    fn on_status(&mut self, data: &[u8]) -> bool {
        println!("on_status: {:?}", str::from_utf8(data).unwrap());
        self.data.extend_from_slice(data);
        true
    }
}

impl ParamHandler for H {}

#[test]
fn response_status_single() {
    let mut h = H{data: Vec::new()};
    let mut p = Parser::new(StreamType::Response);

    assert!(match p.parse(&mut h, b"HTTP/1.1 200 OK") {
        Ok(Success::Eof(_)) => true,
        _ => false
    });

    assert_eq!(h.data, b"OK");
    assert_eq!(p.get_state(), State::ResponseStatus);
}

#[test]
fn response_status_multiple() {
    let mut h = H{data: Vec::new()};
    let mut p = Parser::new(StreamType::Response);

    assert!(match p.parse(&mut h, b"HTTP/1.1 404 NOT FOUND") {
        Ok(Success::Eof(_)) => true,
        _ => false
    });

    assert_eq!(h.data, b"NOT FOUND");
    assert_eq!(p.get_state(), State::ResponseStatus);
}

#[test]
fn response_status_invalid_byte() {
    let mut h = H{data: Vec::new()};
    let mut p = Parser::new(StreamType::Response);

    assert!(match p.parse(&mut h, b"HTTP/1.1 404 NOT@FOUND") {
        Err(ParserError::Status(_,_)) => true,
        _ => false
    });

    assert_eq!(p.get_state(), State::Dead);
}

#[test]
fn response_status_to_body() {
    let mut h = H{data: Vec::new()};
    let mut p = Parser::new(StreamType::Response);

    assert!(match p.parse(&mut h, b"HTTP/1.1 200 OK\r") {
        Ok(Success::Eof(_)) => true,
        _ => false
    });

    assert_eq!(p.get_state(), State::PreHeaders1);

    assert!(match p.parse(&mut h, b"\n") {
        Ok(Success::Eof(_)) => true,
        _ => false
    });

    assert_eq!(p.get_state(), State::PreHeaders2);

    assert!(match p.parse(&mut h, b"\r") {
        Ok(Success::Eof(_)) => true,
        _ => false
    });

    assert_eq!(p.get_state(), State::Newline4);

    assert!(match p.parse(&mut h, b"\n") {
        Ok(Success::Eof(_)) => true,
        _ => false
    });

    assert_eq!(p.get_state(), State::Body);
}

#[test]
fn response_status_invalid_crlf() {
    let mut h = H{data: Vec::new()};
    let mut p = Parser::new(StreamType::Response);

    assert!(match p.parse(&mut h, b"HTTP/1.1 200 OK\r\r") {
        Err(ParserError::CrlfSequence(_,_)) => true,
        _ => false
    });

    assert_eq!(p.get_state(), State::Dead);

    p.reset();

    assert!(match p.parse(&mut h, b"HTTP/1.1 200 OK\n") {
        Err(ParserError::Status(_,_)) => true,
        _ => false
    });

    assert_eq!(p.get_state(), State::Dead);
}
