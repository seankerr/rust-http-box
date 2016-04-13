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

use http1::*;
use std::str;

struct H {
    data: Vec<u8>
}

impl HttpHandler for H {
    fn on_header_field(&mut self, data: &[u8]) -> bool {
        println!("on_header_field: {:?}", str::from_utf8(data).unwrap());
        self.data.extend_from_slice(data);
        true
    }
}

#[test]
fn header_field_eof() {
    let mut h = H{data: Vec::new()};
    let mut p = Parser::new(StreamType::Response);

    assert!(match p.parse(&mut h, b"HTTP/1.1 200 OK\r\nContent-Length") {
        Err(ParserError::Eof) => true,
        _                     => false
    });

    assert_eq!(h.data, b"Content-Length");
    assert_eq!(p.get_state(), State::HeaderField);
}

#[test]
fn header_field_complete() {
    let mut h = H{data: Vec::new()};
    let mut p = Parser::new(StreamType::Response);

    assert!(match p.parse(&mut h, b"HTTP/1.1 200 OK\r\nContent-Length:") {
        Err(ParserError::Eof) => true,
        _                     => false
    });

    assert_eq!(h.data, b"Content-Length");
    assert_eq!(p.get_state(), State::StripHeaderValue);
}

#[test]
fn header_field_invalid_byte() {
    let mut h = H{data: Vec::new()};
    let mut p = Parser::new(StreamType::Response);

    assert!(match p.parse(&mut h, b"HTTP/1.1 200 OK\r\nContent@") {
        Err(ParserError::HeaderField(_,_)) => true,
        _                                  => false
    });

    assert_eq!(p.get_state(), State::Dead);

    p.reset();

    assert!(match p.parse(&mut h, b"HTTP/1.1 200 OK\r\nCont\r") {
        Err(ParserError::HeaderField(_,_)) => true,
        _                                  => false
    });

    assert_eq!(p.get_state(), State::Dead);
}
