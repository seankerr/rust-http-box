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

use fsm::*;
use handler::*;
use http1::*;
use test::http1::*;

macro_rules! setup {
    ($parser:expr, $handler:expr) => ({
        setup(&mut $parser, &mut $handler, b"GET / HTTP/1.1\r\nFieldName: Value",
              ParserState::HeaderValue);
    });
}

#[test]
fn finished() {
    let mut h = DebugHttp1Handler::new();
    let mut p = Parser::new();

    setup!(p, h);

    assert_finished(&mut p, &mut h, b"\r\n\r\n", ParserState::Finished, 4);
    assert!(h.headers_finished);
}

#[test]
fn max_headers_length_error() {
    let mut h = DebugHttp1Handler::new();
    let mut p = Parser::new();

    assert!(match p.parse_headers(&mut h, b"GET / HTTP/1.0\r\nField: Value\r\n\r\n", 31) {
        Err(ParserError::MaxHeadersLength) => {
            true
        },
        _ => {
            false
        }
    });
}

#[test]
fn max_headers_length_ok() {
    let mut h = DebugHttp1Handler::new();
    let mut p = Parser::new();

    assert!(match p.parse_headers(&mut h, b"GET / HTTP/1.0\r\nField: Value\r\n\r\n", 32) {
        Ok(Success::Finished(32)) => {
            true
        },
        _ => false
    });
}
