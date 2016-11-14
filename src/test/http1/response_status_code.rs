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
    () => ({
        let mut parser = Parser::new_head(DebugHandler::new());

        assert_eos!(parser,
                    b"HTTP/1.0 ",
                    StripResponseStatusCode);

        parser
    });
}

#[test]
fn byte_check() {
    // invalid bytes
    loop_non_digits(b" \t", |byte| {
        let mut p = setup!();

        assert_error_byte!(p,
                           &[byte],
                           StatusCode,
                           byte);
    });

    // valid bytes
    loop_digits(b"", |byte| {
        let mut p = setup!();

        assert_eos!(p,
                    &[byte],
                    ResponseStatusCode);
    });
}

#[test]
fn callback_exit() {
    struct CallbackHandler;

    impl HttpHandler for CallbackHandler {
        fn on_status_code(&mut self, _code: u16) -> bool {
            false
        }
    }

    let mut p = Parser::new_head(CallbackHandler);

    assert_eos!(p,
                b"HTTP/1.0 ",
                StripResponseStatusCode);

    assert_callback!(p,
                     b"100 ",
                     StripResponseStatus,
                     3);
}

#[test]
fn v0 () {
    let mut p = setup!();

    assert_eos!(p,
                b"0 ",
                StripResponseStatus);

    assert_eq!(p.handler().status_code,
               0);
}

#[test]
fn v999 () {
    let mut p = setup!();

    assert_eos!(p,
                b"999 ",
                StripResponseStatus);

    assert_eq!(p.handler().status_code,
               999);
}

#[test]
fn v1000 () {
    let mut p = setup!();

    assert_error_byte!(p,
                       b"1000",
                       StatusCode,
                       b'0');
}
