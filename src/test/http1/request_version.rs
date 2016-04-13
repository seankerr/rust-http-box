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

struct H {
    major: u16,
    minor: u16
}

impl HttpHandler for H {
    fn on_version(&mut self, major: u16, minor: u16) -> bool {
        println!("on_version: {}.{}", major, minor);
        self.major = major;
        self.minor = minor;
        true
    }
}

#[test]
fn request_version_eof() {
    let mut h = H{major: 0, minor: 0};
    let mut p = Parser::new(StreamType::Request);

    assert!(match p.parse(&mut h, b"GET /path HTTP/1") {
        Err(ParserError::Eof) => true,
        _                     => false
    });

    assert_eq!(p.get_state(), State::RequestVersionMajor);

    p.reset();

    assert!(match p.parse(&mut h, b"GET /path HTTP/1.1") {
        Err(ParserError::Eof) => true,
        _                     => false
    });

    assert_eq!(p.get_state(), State::RequestVersionMinor);
}

#[test]
fn request_version_0_0() {
    let mut h = H{major: 0, minor: 0};
    let mut p = Parser::new(StreamType::Request);

    assert!(match p.parse(&mut h, b"GET /path HTTP/0.0\r") {
        Err(ParserError::Eof) => true,
        _                     => false
    });

    assert_eq!(h.major, 0);
    assert_eq!(h.minor, 0);
    assert_eq!(p.get_state(), State::PreHeaders1);
}

#[test]
fn request_version_1_1() {
    let mut h = H{major: 0, minor: 0};
    let mut p = Parser::new(StreamType::Request);

    assert!(match p.parse(&mut h, b"GET /path HTTP/1.1\r") {
        Err(ParserError::Eof) => true,
        _                     => false
    });

    assert_eq!(h.major, 1);
    assert_eq!(h.minor, 1);
    assert_eq!(p.get_state(), State::PreHeaders1);
}

#[test]
fn request_version_999_999() {
    let mut h = H{major: 0, minor: 0};
    let mut p = Parser::new(StreamType::Request);

    assert!(match p.parse(&mut h, b"GET /path HTTP/999.999\r") {
        Err(ParserError::Eof) => true,
        _                     => false
    });

    assert_eq!(h.major, 999);
    assert_eq!(h.minor, 999);
    assert_eq!(p.get_state(), State::PreHeaders1);
}

#[test]
fn request_version_invalid() {
    let mut h = H{major: 0, minor: 0};
    let mut p = Parser::new(StreamType::Request);

    assert!(match p.parse(&mut h, b"GET /path HTTP/1000") {
        Err(ParserError::Version(_)) => true,
        _                            => false
    });

    assert_eq!(p.get_state(), State::Dead);

    p.reset();

    assert!(match p.parse(&mut h, b"GET /path HTTP/1.1000") {
        Err(ParserError::Version(_)) => true,
        _                            => false
    });

    assert_eq!(p.get_state(), State::Dead);
}

#[test]
fn request_version_invalid_byte() {
    let mut h = H{major: 0, minor: 0};
    let mut p = Parser::new(StreamType::Request);

    assert!(match p.parse(&mut h, b"GET /path HTTP/@") {
        Err(ParserError::Version(_)) => true,
        _                            => false
    });

    assert_eq!(p.get_state(), State::Dead);

    p.reset();

    assert!(match p.parse(&mut h, b"GET /path HTTP/1@") {
        Err(ParserError::Version(_)) => true,
        _                            => false
    });

    assert_eq!(p.get_state(), State::Dead);

    p.reset();

    assert!(match p.parse(&mut h, b"GET /path HTTP/1.@") {
        Err(ParserError::Version(_)) => true,
        _                            => false
    });

    assert_eq!(p.get_state(), State::Dead);
}
