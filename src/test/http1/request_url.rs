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
use handler::*;
use http1::*;
use test::{ loop_control,
            loop_non_control,
            setup,
            vec_eq };
use url::*;

macro_rules! setup {
    ($parser:expr, $handler:expr) => ({
        setup(&mut $parser, &mut $handler, b"GET ", State::StripRequestUrl);
    });
}

#[test]
fn byte_check() {
    // invalid bytes
    loop_control(b" \t", |byte| {
        let mut h = DebugHandler::new();
        let mut p = Parser::new(StreamType::Request);

        setup!(p, h);

        assert!(match p.parse(&mut h, &[byte]) {
            Err(ParserError::Url(_,x)) => {
                assert_eq!(x, byte);
                assert_eq!(p.get_state(), State::Dead);
                true
            },
            _ => false
        });
    });

    // valid bytes
    loop_non_control(b" \t", |byte| {
        let mut h = DebugHandler::new();
        let mut p = Parser::new(StreamType::Request);

        setup!(p, h);

        assert!(match p.parse(&mut h, &[byte]) {
            Ok(Success::Eof(1)) => {
                assert_eq!(p.get_state(), State::RequestUrl);
                true
            },
            _ => false
        });
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

    impl ParamHandler for X {}

    let mut h = X{};
    let mut p = Parser::new(StreamType::Request);

    setup!(p, h);

    assert!(match p.parse(&mut h, b"/") {
        Ok(Success::Callback(1)) => true,
        _ => false
    });
}

#[test]
fn with_schema() {
    let mut h = DebugHandler::new();
    let mut p = Parser::new(StreamType::Request);

    setup!(p, h);

    assert!(match p.parse(&mut h, b"http://host.com:443/path?query_string#fragment ") {
        Ok(Success::Eof(47)) => {
            vec_eq(h.url, b"http://host.com:443/path?query_string#fragment");
            assert_eq!(p.get_state(), State::StripRequestHttp);
            true
        },
        _ => false
    });
}

#[test]
fn without_schema() {
    let mut h = DebugHandler::new();
    let mut p = Parser::new(StreamType::Request);

    setup!(p, h);

    assert!(match p.parse(&mut h, b"/path?query_string#fragment ") {
        Ok(Success::Eof(28)) => {
            vec_eq(h.url, b"/path?query_string#fragment");
            assert_eq!(p.get_state(), State::StripRequestHttp);
            true
        },
        _ => false
    });
}
