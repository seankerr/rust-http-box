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

use handler::debug::*;
use http1::*;
use test::http1::*;

macro_rules! setup {
    ($parser:expr, $handler:expr) => ({
        setup(&mut $parser, &mut $handler, b"GET / ", ParserState::StripRequestHttp);
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

    assert_callback(&mut p, &mut h, b"HTTP/1.0\r", ParserState::PreHeaders1, 9);
}

#[test]
fn http_1_0 () {
    let mut h = DebugHttp1Handler::new();
    let mut p = Parser::new();

    setup!(p, h);

    assert_eos(&mut p, &mut h, b"HTTP/1.0\r", ParserState::PreHeaders1, 9);
    assert_eq!(h.version_major, 1);
    assert_eq!(h.version_minor, 0);
}

#[test]
fn http_1_1 () {
    let mut h = DebugHttp1Handler::new();
    let mut p = Parser::new();

    setup!(p, h);

    assert_eos(&mut p, &mut h, b"HTTP/1.1\r", ParserState::PreHeaders1, 9);
    assert_eq!(h.version_major, 1);
    assert_eq!(h.version_minor, 1);
}

#[test]
fn http_2_0 () {
    let mut h = DebugHttp1Handler::new();
    let mut p = Parser::new();

    setup!(p, h);

    assert_eos(&mut p, &mut h, b"HTTP/2.0\r", ParserState::PreHeaders1, 9);
    assert_eq!(h.version_major, 2);
    assert_eq!(h.version_minor, 0);
}

#[test]
fn h_lower () {
    let mut h = DebugHttp1Handler::new();
    let mut p = Parser::new();

    setup!(p, h);

    assert_eos(&mut p, &mut h, b"h", ParserState::RequestHttp2, 1);
}

#[test]
fn h_upper () {
    let mut h = DebugHttp1Handler::new();
    let mut p = Parser::new();

    setup!(p, h);

    assert_eos(&mut p, &mut h, b"H", ParserState::RequestHttp2, 1);
}

#[test]
fn ht_lower () {
    let mut h = DebugHttp1Handler::new();
    let mut p = Parser::new();

    setup!(p, h);

    assert_eos(&mut p, &mut h, b"h", ParserState::RequestHttp2, 1);
    assert_eos(&mut p, &mut h, b"t", ParserState::RequestHttp3, 1);
}

#[test]
fn ht_upper () {
    let mut h = DebugHttp1Handler::new();
    let mut p = Parser::new();

    setup!(p, h);

    assert_eos(&mut p, &mut h, b"H", ParserState::RequestHttp2, 1);
    assert_eos(&mut p, &mut h, b"T", ParserState::RequestHttp3, 1);
}

#[test]
fn htt_lower () {
    let mut h = DebugHttp1Handler::new();
    let mut p = Parser::new();

    setup!(p, h);

    assert_eos(&mut p, &mut h, b"h", ParserState::RequestHttp2, 1);
    assert_eos(&mut p, &mut h, b"t", ParserState::RequestHttp3, 1);
    assert_eos(&mut p, &mut h, b"t", ParserState::RequestHttp4, 1);
}

#[test]
fn htt_upper () {
    let mut h = DebugHttp1Handler::new();
    let mut p = Parser::new();

    setup!(p, h);

    assert_eos(&mut p, &mut h, b"H", ParserState::RequestHttp2, 1);
    assert_eos(&mut p, &mut h, b"T", ParserState::RequestHttp3, 1);
    assert_eos(&mut p, &mut h, b"T", ParserState::RequestHttp4, 1);
}

#[test]
fn http_lower () {
    let mut h = DebugHttp1Handler::new();
    let mut p = Parser::new();

    setup!(p, h);

    assert_eos(&mut p, &mut h, b"h", ParserState::RequestHttp2, 1);
    assert_eos(&mut p, &mut h, b"t", ParserState::RequestHttp3, 1);
    assert_eos(&mut p, &mut h, b"t", ParserState::RequestHttp4, 1);
    assert_eos(&mut p, &mut h, b"p", ParserState::RequestHttp5, 1);
}

#[test]
fn http_upper () {
    let mut h = DebugHttp1Handler::new();
    let mut p = Parser::new();

    setup!(p, h);

    assert_eos(&mut p, &mut h, b"H", ParserState::RequestHttp2, 1);
    assert_eos(&mut p, &mut h, b"T", ParserState::RequestHttp3, 1);
    assert_eos(&mut p, &mut h, b"T", ParserState::RequestHttp4, 1);
    assert_eos(&mut p, &mut h, b"P", ParserState::RequestHttp5, 1);
}

#[test]
fn http_slash_lower () {
    let mut h = DebugHttp1Handler::new();
    let mut p = Parser::new();

    setup!(p, h);

    assert_eos(&mut p, &mut h, b"h", ParserState::RequestHttp2, 1);
    assert_eos(&mut p, &mut h, b"t", ParserState::RequestHttp3, 1);
    assert_eos(&mut p, &mut h, b"t", ParserState::RequestHttp4, 1);
    assert_eos(&mut p, &mut h, b"p", ParserState::RequestHttp5, 1);
    assert_eos(&mut p, &mut h, b"/", ParserState::RequestVersionMajor, 1);
}

#[test]
fn http_slash_upper () {
    let mut h = DebugHttp1Handler::new();
    let mut p = Parser::new();

    setup!(p, h);

    assert_eos(&mut p, &mut h, b"H", ParserState::RequestHttp2, 1);
    assert_eos(&mut p, &mut h, b"T", ParserState::RequestHttp3, 1);
    assert_eos(&mut p, &mut h, b"T", ParserState::RequestHttp4, 1);
    assert_eos(&mut p, &mut h, b"P", ParserState::RequestHttp5, 1);
    assert_eos(&mut p, &mut h, b"/", ParserState::RequestVersionMajor, 1);
}
