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
        let mut parser = Parser::new_head(DebugHandler::new());

        assert_eos!(parser,
                    b"GET / HTTP/",
                    RequestVersionMajor);

        parser
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

    let mut p = Parser::new_head(CallbackHandler);

    assert_eos!(p,
                b"GET / HTTP/",
                RequestVersionMajor);

    assert_callback!(p,
                     b"1.0\r",
                     InitialEnd);
}

#[test]
fn v0_0 () {
    let mut p = setup!();

    assert_eos!(p,
                b"0.0\r",
                PreHeadersLf1);

    assert_eq!(p.handler().version_major,
               0);

    assert_eq!(p.handler().version_minor,
               0);
}

#[test]
fn v1_0 () {
    let mut p = setup!();

    assert_eos!(p,
                b"1.0\r",
                PreHeadersLf1);

    assert_eq!(p.handler().version_major,
               1);

    assert_eq!(p.handler().version_minor,
               0);
}

#[test]
fn v1_1 () {
    let mut p = setup!();

    assert_eos!(p,
                b"1.1\r",
                PreHeadersLf1);

    assert_eq!(p.handler().version_major,
               1);

    assert_eq!(p.handler().version_minor,
               1);
}

#[test]
fn v2_0 () {
    let mut p = setup!();

    assert_eos!(p,
                b"2.0\r",
                PreHeadersLf1);

    assert_eq!(p.handler().version_major,
               2);

    assert_eq!(p.handler().version_minor,
               0);
}

#[test]
fn v999_999 () {
    let mut p = setup!();

    assert_eos!(p,
                b"999.999\r",
                PreHeadersLf1);

    assert_eq!(p.handler().version_major,
               999);

    assert_eq!(p.handler().version_minor,
               999);
}

#[test]
fn v1000_0 () {
    let mut p = setup!();

    assert_error_byte!(p,
                       b"1000",
                       Version,
                       b'0');
}

#[test]
fn v0_1000 () {
    let mut p = setup!();

    assert_error_byte!(p,
                       b"0.1000",
                       Version,
                       b'0');
}
