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
// | Author: Sean Kerr <sean@metatomic.io>                                                         |
// +-----------------------------------------------------------------------------------------------+

use http1::*;
use test::*;
use test::http1::*;

macro_rules! setup {
    () => ({
        let mut handler = DebugHandler::new();
        let mut parser  = Parser::new_head();

        assert_eos!(
            parser,
            handler,
            b"GET ",
            StripRequestUrl
        );

        (parser, handler)
    });
}

#[test]
fn asterisk() {
    let (mut p, mut h) = setup!();

    assert_eos!(
        p,
        h,
        b"* ",
        StripRequestHttp
    );
}

#[test]
fn byte_check() {
    // invalid bytes
    loop_non_visible(b" \t", |byte| {
        let (mut p, mut h) = setup!();

        assert_error_byte!(
            p,
            h,
            &[byte],
            Url,
            byte
        );
    });

    // valid bytes
    loop_visible(b"", |byte| {
        let (mut p, mut h) = setup!();

        assert_eos!(
            p,
            h,
            &[byte],
            RequestUrl
        );
    });
}

#[test]
fn callback_exit() {
    struct CallbackHandler;

    impl HttpHandler for CallbackHandler {
        fn on_url(&mut self, _url: &[u8]) -> bool {
            false
        }
    }

    let mut h = CallbackHandler;
    let mut p = Parser::new_head();

    assert_eos!(
        p,
        h,
        b"GET ",
        StripRequestUrl
    );

    assert_callback!(
        p,
        h,
        b"/",
        RequestUrl
    );
}

#[test]
fn with_schema() {
    let (mut p, mut h) = setup!();

    assert_eos!(
        p,
        h,
        b"http://host.com:443/path?query_string#fragment ",
        StripRequestHttp
    );

    vec_eq(
        b"http://host.com:443/path?query_string#fragment",
        &h.url
    );
}

#[test]
fn without_schema() {
    let (mut p, mut h) = setup!();

    assert_eos!(
        p,
        h,
        b"/path?query_string#fragment ",
        StripRequestHttp
    );

    vec_eq(
        b"/path?query_string#fragment",
        &h.url
    );
}
