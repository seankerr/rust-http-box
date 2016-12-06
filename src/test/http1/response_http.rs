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
use test::http1::*;

macro_rules! setup {
    () => ({
        Parser::new_head(DebugHandler::new())
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

    assert_callback!(p,
                     b"HTTP/1.0 ",
                     StripResponseStatusCode);
}

#[test]
fn http_1_0 () {
    let mut p = setup!();

    assert_eos!(p,
                b"HTTP/1.0 ",
                StripResponseStatusCode);

    assert_eq!(p.handler().version_major,
               1);

    assert_eq!(p.handler().version_minor,
               0);
}

#[test]
fn http_1_1 () {
    let mut p = setup!();

    assert_eos!(p,
                b"HTTP/1.1 ",
                StripResponseStatusCode);

    assert_eq!(p.handler().version_major,
               1);

    assert_eq!(p.handler().version_minor,
               1);
}

#[test]
fn http_2_0 () {
    let mut p = setup!();

    assert_eos!(p,
                b"HTTP/2.0 ",
                StripResponseStatusCode);

    assert_eq!(p.handler().version_major,
               2);

    assert_eq!(p.handler().version_minor,
               0);
}

#[test]
fn h_lower () {
    let mut p = setup!();

    assert_eos!(p,
                b"h",
                Detect2);
}

#[test]
fn h_upper () {
    let mut p = setup!();

    assert_eos!(p,
                b"H",
                Detect2);
}

#[test]
fn ht_lower () {
    let mut p = setup!();

    assert_eos!(p,
                b"h",
                Detect2);

    assert_eos!(p,
                b"t",
                Detect3);
}

#[test]
fn ht_upper () {
    let mut p = setup!();

    assert_eos!(p,
                b"H",
                Detect2);

    assert_eos!(p,
                b"T",
                Detect3);
}

#[test]
fn htt_lower () {
    let mut p = setup!();

    assert_eos!(p,
                b"h",
                Detect2);

    assert_eos!(p,
                b"t",
                Detect3);

    assert_eos!(p,
                b"t",
                Detect4);
}

#[test]
fn htt_upper () {
    let mut p = setup!();

    assert_eos!(p,
                b"H",
                Detect2);

    assert_eos!(p,
                b"T",
                Detect3);

    assert_eos!(p,
                b"T",
                Detect4);
}

#[test]
fn http_lower () {
    let mut p = setup!();

    assert_eos!(p,
                b"h",
                Detect2);

    assert_eos!(p,
                b"t",
                Detect3);

    assert_eos!(p,
                b"t",
                Detect4);

    assert_eos!(p,
                b"p",
                Detect5);
}

#[test]
fn http_upper () {
    let mut p = setup!();

    assert_eos!(p,
                b"H",
                Detect2);

    assert_eos!(p,
                b"T",
                Detect3);

    assert_eos!(p,
                b"T",
                Detect4);

    assert_eos!(p,
                b"P",
                Detect5);
}

#[test]
fn http_slash_lower () {
    let mut p = setup!();

    assert_eos!(p,
                b"h",
                Detect2);

    assert_eos!(p,
                b"t",
                Detect3);

    assert_eos!(p,
                b"t",
                Detect4);

    assert_eos!(p,
                b"p",
                Detect5);

    assert_eos!(p,
                b"/",
                ResponseVersionMajor);
}

#[test]
fn http_slash_upper () {
    let mut p = setup!();

    assert_eos!(p,
                b"H",
                Detect2);

    assert_eos!(p,
                b"T",
                Detect3);

    assert_eos!(p,
                b"T",
                Detect4);

    assert_eos!(p,
                b"P",
                Detect5);

    assert_eos!(p,
                b"/",
                ResponseVersionMajor);
}
