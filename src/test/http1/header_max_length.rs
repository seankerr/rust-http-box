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

struct H {}

impl HttpHandler for H {}

#[test]
fn header_max_length() {
    let mut h = H{};
    let mut p = Parser::with_settings(StreamType::Response, 1);

    assert!(match p.parse(&mut h, b"H") {
        Err(ParserError::Eof) => true,
        _                     => false
    });

    assert_eq!(p.get_state(), State::ResponseHttp2);

    assert!(match p.parse(&mut h, b"T") {
        Err(ParserError::MaxHeadersLength(_,x)) => { assert_eq!(x, 1); true },
        _                                       => false
    });

    assert_eq!(p.get_state(), State::Dead);

    p.reset();

    assert!(match p.parse(&mut h, b"HT") {
        Err(ParserError::MaxHeadersLength(_,x)) => { assert_eq!(x, 1); true },
        _                                       => false
    });

    assert_eq!(p.get_state(), State::Dead);
}

#[test]
fn header_max_length_multiple_stream() {
    let mut h = H{};
    let mut p = Parser::with_settings(StreamType::Response, 3);

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
        Err(ParserError::MaxHeadersLength(_,_)) => true,
        _                                       => false
    });

    assert_eq!(p.get_state(), State::Dead);
}

#[test]
fn header_max_length_advance_byte() {
    let mut h = H{};
    let mut p = Parser::with_settings(StreamType::Response, 5);

    assert!(match p.parse(&mut h, b"HTTP/") {
        Err(ParserError::Eof) => true,
        _                     => false
    });

    assert_eq!(p.get_state(), State::ResponseVersionMajor);

    p.reset();

    assert!(match p.parse(&mut h, b"HTTP/1") {
        Err(ParserError::MaxHeadersLength(_,_)) => true,
        _                                       => false
    });

    assert_eq!(p.get_state(), State::Dead);
}
