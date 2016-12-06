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
        Parser::new_head(DebugHandler::new())
    });
}

#[test]
fn byte_check() {
    // invalid bytes
    loop_non_tokens(b"\r\n\t ", |byte| {
        let mut p = setup!();

        assert_error_byte!(p,
                           &[byte],
                           Method,
                           byte);
    });

    // valid lower-cased alphabetical bytes
    loop_tokens(b"Hh", |byte| {
        if byte > 0x60 && byte < 0x7B {
            let mut p = setup!();

            assert_eos!(p,
                        &[byte],
                        LowerRequestMethod);
        }
    });

    // valid upper-cased alphabetical bytes
    loop_tokens(b"Hh", |byte| {
        if !(byte > 0x60 && byte < 0x7B) {
            let mut p = setup!();

            assert_eos!(p,
                        &[byte],
                        UpperRequestMethod);
        }
    });

    for n in &[b'H', b'h'] {
        // valid H|h byte
        let mut p = setup!();

        assert_eos!(p,
                    &[*n as u8],
                    Detect2);
    }
}

#[test]
fn callback_exit() {
    struct CallbackHandler;

    impl HttpHandler for CallbackHandler {
        fn on_method(&mut self, _method: &[u8]) -> bool {
            false
        }
    }

    let mut p = Parser::new_head(CallbackHandler);

    assert_callback!(p,
                     b"G",
                     UpperRequestMethod);
}

#[test]
fn multiple_connect() {
    let mut p = setup!();

    assert_eos!(p,
               b"C",
               UpperRequestMethod);

    assert_eq!(p.handler().method,
               b"C");

    assert_eos!(p,
               b"O",
               UpperRequestMethod);

    assert_eq!(p.handler().method,
               b"CO");

    assert_eos!(p,
               b"N",
               UpperRequestMethod);

    assert_eq!(p.handler().method,
               b"CON");

    assert_eos!(p,
               b"N",
               UpperRequestMethod);

    assert_eq!(p.handler().method,
               b"CONN");

    assert_eos!(p,
               b"E",
               UpperRequestMethod);

    assert_eq!(p.handler().method,
               b"CONNE");

    assert_eos!(p,
               b"C",
               UpperRequestMethod);

    assert_eq!(p.handler().method,
               b"CONNEC");

    assert_eos!(p,
               b"T",
               UpperRequestMethod);

    assert_eq!(p.handler().method,
               b"CONNECT");

    assert_eos!(p,
               b" ",
               StripRequestUrl);

    assert_eq!(p.handler().method,
               b"CONNECT");
}

#[test]
fn multiple_delete() {
    let mut p = setup!();

    assert_eos!(p,
                b"D",
                UpperRequestMethod);

    assert_eq!(p.handler().method,
               b"D");

    assert_eos!(p,
                b"E",
                UpperRequestMethod);

    assert_eq!(p.handler().method,
               b"DE");

    assert_eos!(p,
                b"L",
                UpperRequestMethod);

    assert_eq!(p.handler().method,
               b"DEL");

    assert_eos!(p,
                b"E",
                UpperRequestMethod);

    assert_eq!(p.handler().method,
               b"DELE");

    assert_eos!(p,
                b"T",
                UpperRequestMethod);

    assert_eq!(p.handler().method,
               b"DELET");

    assert_eos!(p,
                b"E",
                UpperRequestMethod);

    assert_eq!(p.handler().method,
               b"DELETE");

    assert_eos!(p,
                b" ",
                StripRequestUrl);

    assert_eq!(p.handler().method,
               b"DELETE");
}

#[test]
fn multiple_get() {
    let mut p = setup!();

    assert_eos!(p,
                b"G",
                UpperRequestMethod);

    assert_eq!(p.handler().method,
               b"G");

    assert_eos!(p,
                b"E",
                UpperRequestMethod);

    assert_eq!(p.handler().method,
               b"GE");

    assert_eos!(p,
                b"T",
                UpperRequestMethod);

    assert_eq!(p.handler().method,
               b"GET");

    assert_eos!(p,
                b" ",
                StripRequestUrl);

    assert_eq!(p.handler().method,
               b"GET");
}

#[test]
fn multiple_head() {
    let mut p = setup!();

    assert_eos!(p,
                b"H",
                Detect2);

    assert_eos!(p,
                b"E",
                UpperRequestMethod);

    assert_eq!(p.handler().method,
               b"HE");

    assert_eos!(p,
                b"A",
                UpperRequestMethod);

    assert_eq!(p.handler().method,
               b"HEA");

    assert_eos!(p,
                b"D",
                UpperRequestMethod);

    assert_eq!(p.handler().method,
               b"HEAD");

    assert_eos!(p,
                b" ",
                StripRequestUrl);

    assert_eq!(p.handler().method,
               b"HEAD");
}

#[test]
fn multiple_options() {
    let mut p = setup!();

    assert_eos!(p,
                b"O",
                UpperRequestMethod);

    assert_eq!(p.handler().method,
               b"O");

    assert_eos!(p,
                b"P",
                UpperRequestMethod);

    assert_eq!(p.handler().method,
               b"OP");

    assert_eos!(p,
                b"T",
                UpperRequestMethod);

    assert_eq!(p.handler().method,
               b"OPT");

    assert_eos!(p,
                b"I",
                UpperRequestMethod);

    assert_eq!(p.handler().method,
               b"OPTI");

    assert_eos!(p,
                b"O",
                UpperRequestMethod);

    assert_eq!(p.handler().method,
               b"OPTIO");

    assert_eos!(p,
                b"N",
                UpperRequestMethod);

    assert_eq!(p.handler().method,
               b"OPTION");

    assert_eos!(p,
                b"S",
                UpperRequestMethod);

    assert_eq!(p.handler().method,
               b"OPTIONS");

    assert_eos!(p,
                b" ",
                StripRequestUrl);

    assert_eq!(p.handler().method,
               b"OPTIONS");
}

#[test]
fn multiple_post() {
    let mut p = setup!();

    assert_eos!(p,
                b"P",
                UpperRequestMethod);

    assert_eq!(p.handler().method,
               b"P");

    assert_eos!(p,
                b"O",
                UpperRequestMethod);

    assert_eq!(p.handler().method,
               b"PO");

    assert_eos!(p,
                b"S",
                UpperRequestMethod);

    assert_eq!(p.handler().method,
               b"POS");

    assert_eos!(p,
                b"T",
                UpperRequestMethod);

    assert_eq!(p.handler().method,
               b"POST");

    assert_eos!(p,
                b" ",
                StripRequestUrl);

    assert_eq!(p.handler().method,
               b"POST");
}

#[test]
fn multiple_put() {
    let mut p = setup!();

    assert_eos!(p,
                b"P",
                UpperRequestMethod);

    assert_eq!(p.handler().method,
               b"P");

    assert_eos!(p,
                b"U",
                UpperRequestMethod);

    assert_eq!(p.handler().method,
               b"PU");

    assert_eos!(p,
                b"T",
                UpperRequestMethod);

    assert_eq!(p.handler().method,
               b"PUT");

    assert_eos!(p,
                b" ",
                StripRequestUrl);

    assert_eq!(p.handler().method,
               b"PUT");
}

#[test]
fn multiple_trace() {
    let mut p = setup!();

    assert_eos!(p,
                b"T",
                UpperRequestMethod);

    assert_eq!(p.handler().method,
               b"T");

    assert_eos!(p,
                b"R",
                UpperRequestMethod);

    assert_eq!(p.handler().method,
               b"TR");

    assert_eos!(p,
                b"A",
                UpperRequestMethod);

    assert_eq!(p.handler().method,
               b"TRA");

    assert_eos!(p,
                b"C",
                UpperRequestMethod);

    assert_eq!(p.handler().method,
               b"TRAC");

    assert_eos!(p,
                b"E",
                UpperRequestMethod);

    assert_eq!(p.handler().method,
               b"TRACE");

    assert_eos!(p,
                b" ",
                StripRequestUrl);

    assert_eq!(p.handler().method,
               b"TRACE");
}

#[test]
fn multiple_unknown() {
    let mut p = setup!();

    assert_eos!(p,
                b"U",
                UpperRequestMethod);

    assert_eq!(p.handler().method,
               b"U");

    assert_eos!(p,
                b"N",
                UpperRequestMethod);

    assert_eq!(p.handler().method,
               b"UN");

    assert_eos!(p,
                b"K",
                UpperRequestMethod);

    assert_eq!(p.handler().method,
               b"UNK");

    assert_eos!(p,
                b"N",
                UpperRequestMethod);

    assert_eq!(p.handler().method,
               b"UNKN");

    assert_eos!(p,
                b"O",
                UpperRequestMethod);

    assert_eq!(p.handler().method,
               b"UNKNO");

    assert_eos!(p,
                b"W",
                UpperRequestMethod);

    assert_eq!(p.handler().method,
               b"UNKNOW");

    assert_eos!(p,
                b"N",
                UpperRequestMethod);

    assert_eq!(p.handler().method,
               b"UNKNOWN");

    assert_eos!(p,
                b" ",
                StripRequestUrl);

    assert_eq!(p.handler().method,
               b"UNKNOWN");
}

#[test]
fn normalize() {
    let mut p = setup!();

    assert_eos!(p,
                b"g",
                LowerRequestMethod);

    assert_eq!(p.handler().method,
               b"G");

    assert_eos!(p,
                b"E",
                UpperRequestMethod);

    assert_eq!(p.handler().method,
               b"GE");

    assert_eos!(p,
                b"t",
                LowerRequestMethod);

    assert_eq!(p.handler().method,
               b"GET");

    assert_eos!(p,
                b" ",
                StripRequestUrl);

    assert_eq!(p.handler().method,
               b"GET");
}

#[test]
fn single_connect() {
    let mut p = setup!();

    assert_eos!(p,
                b"CONNECT ",
                StripRequestUrl);

    assert_eq!(p.handler().method,
               b"CONNECT");
}

#[test]
fn single_delete() {
    let mut p = setup!();

    assert_eos!(p,
                b"DELETE  ",
                StripRequestUrl);

    assert_eq!(p.handler().method,
               b"DELETE");
}

#[test]
fn single_get() {
    let mut p = setup!();

    assert_eos!(p,
                b"GET     ",
                StripRequestUrl);

    assert_eq!(p.handler().method,
               b"GET");
}

#[test]
fn single_head() {
    let mut p = setup!();

    assert_eos!(p,
                b"HEAD    ",
                StripRequestUrl);

    assert_eq!(p.handler().method,
               b"HEAD");
}

#[test]
fn single_options() {
    let mut p = setup!();

    assert_eos!(p,
                b"OPTIONS ",
                StripRequestUrl);

    assert_eq!(p.handler().method,
               b"OPTIONS");
}

#[test]
fn single_post() {
    let mut p = setup!();

    assert_eos!(p,
                b"POST    ",
                StripRequestUrl);

    assert_eq!(p.handler().method,
               b"POST");
}

#[test]
fn single_put() {
    let mut p = setup!();

    assert_eos!(p,
                b"PUT     ",
                StripRequestUrl);

    assert_eq!(p.handler().method,
               b"PUT");
}

#[test]
fn single_trace() {
    let mut p = setup!();

    assert_eos!(p,
                b"TRACE   ",
                StripRequestUrl);

    assert_eq!(p.handler().method,
               b"TRACE");
}

#[test]
fn single_unknown() {
    let mut p = setup!();

    assert_eos!(p,
                b"UNKNOWN ",
                StripRequestUrl);

    assert_eq!(p.handler().method,
               b"UNKNOWN");
}

#[test]
fn starting_space() {
    let mut p = setup!();

    assert_eos!(p,
                b"   ",
                StripDetect);
}
