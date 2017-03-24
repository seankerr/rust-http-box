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
use test::http1::*;

macro_rules! setup {
    () => ({
        let mut handler = DebugHandler::new();
        let mut parser  = Parser::new();

        assert_eos!(
            parser,
            handler,
            b"GET / ",
            StripRequestHttp
        );

        (parser, handler)
    });
}

#[test]
fn callback_exit() {
    struct CallbackHandler;

    impl HttpHandler for CallbackHandler {
        fn on_version(&mut self, _major: u16, _minor: u16) -> bool {
            false
        }
    }

    let mut h = CallbackHandler;
    let mut p = Parser::new();

    assert_eos!(
        p,
        h,
        b"GET / ",
        StripRequestHttp
    );

    assert_callback!(
        p,
        h,
        b"HTTP/1.0\r",
        InitialEnd
    );
}

#[test]
fn http_1_0 () {
    let (mut p, mut h) = setup!();

    assert_eos!(
        p,
        h,
        b"HTTP/1.0\r",
        PreHeadersLf1
    );

    assert_eq!(
        h.version_major,
        1
    );

    assert_eq!(
        h.version_minor,
        0
    );
}

#[test]
fn http_1_1 () {
    let (mut p, mut h) = setup!();

    assert_eos!(
        p,
        h,
        b"HTTP/1.1\r",
        PreHeadersLf1
    );

    assert_eq!(
        h.version_major,
        1
    );

    assert_eq!(
        h.version_minor,
        1
    );
}

#[test]
fn http_2_0 () {
    let (mut p, mut h) = setup!();

    assert_eos!(
        p,
        h,
        b"HTTP/2.0\r",
        PreHeadersLf1
    );

    assert_eq!(
        h.version_major,
        2
    );

    assert_eq!(
        h.version_minor,
        0
    );
}

#[test]
fn h_lower () {
    let (mut p, mut h) = setup!();

    assert_eos!(
        p,
        h,
        b"h",
        RequestHttp2
    );
}

#[test]
fn h_upper () {
    let (mut p, mut h) = setup!();

    assert_eos!(
        p,
        h,
        b"H",
        RequestHttp2
    );
}

#[test]
fn ht_lower () {
    let (mut p, mut h) = setup!();

    assert_eos!(
        p,
        h,
        b"h",
        RequestHttp2
    );

    assert_eos!(
        p,
        h,
        b"t",
        RequestHttp3
    );
}

#[test]
fn ht_upper () {
    let (mut p, mut h) = setup!();

    assert_eos!(
        p,
        h,
        b"H",
        RequestHttp2
    );

    assert_eos!(
        p,
        h,
        b"T",
        RequestHttp3
    );
}

#[test]
fn htt_lower () {
    let (mut p, mut h) = setup!();

    assert_eos!(
        p,
        h,
        b"h",
        RequestHttp2
    );

    assert_eos!(
        p,
        h,
        b"t",
        RequestHttp3
    );

    assert_eos!(
        p,
        h,
        b"t",
        RequestHttp4
    );
}

#[test]
fn htt_upper () {
    let (mut p, mut h) = setup!();

    assert_eos!(
        p,
        h,
        b"H",
        RequestHttp2
    );

    assert_eos!(
        p,
        h,
        b"T",
        RequestHttp3
    );

    assert_eos!(
        p,
        h,
        b"T",
        RequestHttp4
    );
}

#[test]
fn http_lower () {
    let (mut p, mut h) = setup!();

    assert_eos!(
        p,
        h,
        b"h",
        RequestHttp2
    );

    assert_eos!(
        p,
        h,
        b"t",
        RequestHttp3
    );

    assert_eos!(
        p,
        h,
        b"t",
        RequestHttp4
    );

    assert_eos!(
        p,
        h,
        b"p",
        RequestHttp5
    );
}

#[test]
fn http_upper () {
    let (mut p, mut h) = setup!();

    assert_eos!(
        p,
        h,
        b"H",
        RequestHttp2
    );

    assert_eos!(
        p,
        h,
        b"T",
        RequestHttp3
    );

    assert_eos!(
        p,
        h,
        b"T",
        RequestHttp4
    );

    assert_eos!(
        p,
        h,
        b"P",
        RequestHttp5
    );
}

#[test]
fn http_slash_lower () {
    let (mut p, mut h) = setup!();

    assert_eos!(
        p,
        h,
        b"h",
        RequestHttp2
    );

    assert_eos!(
        p,
        h,
        b"t",
        RequestHttp3
    );

    assert_eos!(
        p,
        h,
        b"t",
        RequestHttp4
    );

    assert_eos!(
        p,
        h,
        b"p",
        RequestHttp5
    );

    assert_eos!(
        p,
        h,
        b"/",
        RequestVersionMajor1
    );
}

#[test]
fn http_slash_upper () {
    let (mut p, mut h) = setup!();

    assert_eos!(
        p,
        h,
        b"H",
        RequestHttp2
    );

    assert_eos!(
        p,
        h,
        b"T",
        RequestHttp3
    );

    assert_eos!(
        p,
        h,
        b"T",
        RequestHttp4
    );

    assert_eos!(
        p,
        h,
        b"P",
        RequestHttp5
    );

    assert_eos!(
        p,
        h,
        b"/",
        RequestVersionMajor1
    );
}
