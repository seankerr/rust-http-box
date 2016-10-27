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

use extra::handler::*;
use http1::*;
use test::http1::*;

macro_rules! setup {
    ($parser:expr, $handler:expr) => ({
        setup(&mut $parser, &mut $handler, b"GET / HTTP/", ParserState::RequestVersionMajor);
    });
}

#[test]
fn callback_exit() {
    struct X;

    impl Http1Handler for X {
        fn on_version(&mut self, _major: u16, _minor: u16) -> bool {
            false
        }
    }

    let mut h = X{};
    let mut p = Parser::new();

    setup!(p, h);

    assert_callback(&mut p, &mut h, b"1.0\r", ParserState::StatusEnd, 4);
}

#[test]
fn v0_0 () {
    let mut h = DebugHttp1Handler::new();
    let mut p = Parser::new();

    setup!(p, h);

    assert_eos(&mut p, &mut h, b"0.0\r", ParserState::PreHeaders1, 4);
    assert_eq!(h.version_major, 0);
    assert_eq!(h.version_minor, 0);
}

#[test]
fn v1_0 () {
    let mut h = DebugHttp1Handler::new();
    let mut p = Parser::new();

    setup!(p, h);

    assert_eos(&mut p, &mut h, b"1.0\r", ParserState::PreHeaders1, 4);
    assert_eq!(h.version_major, 1);
    assert_eq!(h.version_minor, 0);
}

#[test]
fn v1_1 () {
    let mut h = DebugHttp1Handler::new();
    let mut p = Parser::new();

    setup!(p, h);

    assert_eos(&mut p, &mut h, b"1.1\r", ParserState::PreHeaders1, 4);
    assert_eq!(h.version_major, 1);
    assert_eq!(h.version_minor, 1);
}

#[test]
fn v2_0 () {
    let mut h = DebugHttp1Handler::new();
    let mut p = Parser::new();

    setup!(p, h);

    assert_eos(&mut p, &mut h, b"2.0\r", ParserState::PreHeaders1, 4);
    assert_eq!(h.version_major, 2);
    assert_eq!(h.version_minor, 0);
}

#[test]
fn v999_999 () {
    let mut h = DebugHttp1Handler::new();
    let mut p = Parser::new();

    setup!(p, h);

    assert_eos(&mut p, &mut h, b"999.999\r", ParserState::PreHeaders1, 8);
    assert_eq!(h.version_major, 999);
    assert_eq!(h.version_minor, 999);
}

#[test]
fn v1000_0 () {
    let mut h = DebugHttp1Handler::new();
    let mut p = Parser::new();

    setup!(p, h);

    if let ParserError::Version(x) = assert_error(&mut p, &mut h, b"1000").unwrap() {
        assert_eq!(x, b'0');
    } else {
        panic!();
    }
}

#[test]
fn v0_1000 () {
    let mut h = DebugHttp1Handler::new();
    let mut p = Parser::new();

    setup!(p, h);

    if let ParserError::Version(x) = assert_error(&mut p, &mut h, b"0.1000").unwrap() {
        assert_eq!(x, b'0');
    } else {
        panic!();
    }
}
