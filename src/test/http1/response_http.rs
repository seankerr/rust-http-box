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

struct H {}

impl HttpHandler for H {}

#[test]
fn response_http_eof() {
    let mut h = H{};
    let mut p = Parser::new(StreamType::Response);

    assert!(match p.parse(&mut h, b"HTTP") {
        Err(ParserError::Eof) => true,
        _                     => false
    });

    assert_eq!(p.get_state(), State::ResponseHttp5);
}

#[test]
fn response_http_upper() {
    let mut h = H{};
    let mut p = Parser::new(StreamType::Response);

    assert!(match p.parse(&mut h, b"HTTP/") {
        Err(ParserError::Eof) => true,
        _                     => false
    });

    assert_eq!(p.get_state(), State::ResponseVersionMajor);
}

#[test]
fn response_http_lower() {
    let mut h = H{};
    let mut p = Parser::new(StreamType::Response);

    assert!(match p.parse(&mut h, b"http/") {
        Err(ParserError::Eof) => true,
        _                     => false
    });

    assert_eq!(p.get_state(), State::ResponseVersionMajor);
}

#[test]
fn response_http_multiple_streams() {
    let mut h = H{};
    let mut p = Parser::new(StreamType::Response);

    assert!(match p.parse(&mut h, b"H") {
        Err(ParserError::Eof) => true,
        _                     => false
    });

    assert_eq!(p.get_state(), State::ResponseHttp2);

    assert!(match p.parse(&mut h, b"T") {
        Err(ParserError::Eof) => true,
        _                     => false
    });

    assert_eq!(p.get_state(), State::ResponseHttp3);

    assert!(match p.parse(&mut h, b"T") {
        Err(ParserError::Eof) => true,
        _                     => false
    });

    assert_eq!(p.get_state(), State::ResponseHttp4);

    assert!(match p.parse(&mut h, b"P") {
        Err(ParserError::Eof) => true,
        _                     => false
    });

    assert_eq!(p.get_state(), State::ResponseHttp5);

    assert!(match p.parse(&mut h, b"/") {
        Err(ParserError::Eof) => true,
        _                     => false
    });

    assert_eq!(p.get_state(), State::ResponseVersionMajor);
}

#[test]
fn response_http_invalid_byte() {
    let mut h = H{};
    let mut p = Parser::new(StreamType::Response);

    assert!(match p.parse(&mut h, b"HTT@/") {
        Err(ParserError::Version(_)) => true,
        _                            => false
    });

    assert_eq!(p.get_state(), State::Dead);
}
