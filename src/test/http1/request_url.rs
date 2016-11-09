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
use test::*;
use test::http1::*;

macro_rules! setup {
    ($parser:expr, $handler:expr) => ({
        $parser.init_head();

        assert_eos!($parser, $handler,
                    b"GET ",
                    ParserState::StripRequestUrl);
    });
}

#[test]
fn byte_check() {
    // invalid bytes
    loop_non_visible(b" \t", |byte| {
        let mut h = DebugHandler::new();
        let mut p = Parser::new();

        setup!(p, h);

        assert_error_byte!(p, h,
                           &[byte],
                           ParserError::Url,
                           byte);
    });

    // valid bytes
    loop_visible(b"", |byte| {
        let mut h = DebugHandler::new();
        let mut p = Parser::new();

        setup!(p, h);

        assert_eos!(p, h,
                    &[byte],
                    ParserState::RequestUrl);
    });
}

#[test]
fn callback_exit() {
    struct X;

    impl HttpHandler for X {
        fn on_url(&mut self, _url: &[u8]) -> bool {
            false
        }
    }

    let mut h = X{};
    let mut p = Parser::new();

    setup!(p, h);

    assert_callback!(p, h,
                     b"/",
                     ParserState::RequestUrl);
}

#[test]
fn with_schema() {
    let mut h = DebugHandler::new();
    let mut p = Parser::new();

    setup!(p, h);

    assert_eos!(p, h,
                b"http://host.com:443/path?query_string#fragment ",
                ParserState::StripRequestHttp);
    vec_eq(&h.url, b"http://host.com:443/path?query_string#fragment");
}

#[test]
fn without_schema() {
    let mut h = DebugHandler::new();
    let mut p = Parser::new();

    setup!(p, h);

    assert_eos!(p, h,
                b"/path?query_string#fragment ",
                ParserState::StripRequestHttp);
    vec_eq(&h.url, b"/path?query_string#fragment");
}
