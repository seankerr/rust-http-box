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

use http1::parser::*;

struct H {
    data: u16
}

impl HttpHandler for H {
    fn on_status_code(&mut self, data: u16) -> bool {
        self.data = data;
        true
    }
}

#[test]
fn response_status_code_eof() {
    let mut h = H{data: 0};
    let mut p = Parser::new(StreamType::Response);

    assert!(match p.parse(&mut h, b"HTTP/1.1 0") {
        Err(ParserError::Eof) => true,
        _                     => false
    });

    assert_eq!(p.get_state(), State::ResponseStatusCode);
}

#[test]
fn response_status_code_0() {
    let mut h = H{data: 0};
    let mut p = Parser::new(StreamType::Response);

    assert!(match p.parse(&mut h, b"HTTP/1.1 0 ") {
        Err(ParserError::Eof) => true,
        _                     => false
    });

    assert_eq!(h.data, 0);
    assert_eq!(p.get_state(), State::ResponseStatus);
}

#[test]
fn response_status_code_999() {
    let mut h = H{data: 0};
    let mut p = Parser::new(StreamType::Response);

    assert!(match p.parse(&mut h, b"HTTP/1.1 999 ") {
        Err(ParserError::Eof) => true,
        _                     => false
    });

    assert_eq!(h.data, 999);
    assert_eq!(p.get_state(), State::ResponseStatus);
}

#[test]
fn response_status_code_invalid() {
    let mut h = H{data: 0};
    let mut p = Parser::new(StreamType::Response);

    assert!(match p.parse(&mut h, b"HTTP/1.1 1000") {
        Err(ParserError::StatusCode(_)) => true,
        _                               => false
    });

    assert_eq!(p.get_state(), State::Dead);
}

#[test]
fn response_status_code_invalid_byte() {
    let mut h = H{data: 0};
    let mut p = Parser::new(StreamType::Response);

    assert!(match p.parse(&mut h, b"HTTP/1.1 a") {
        Err(ParserError::StatusCode(_)) => true,
        _                               => false
    });

    assert_eq!(p.get_state(), State::Dead);
}
