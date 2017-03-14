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
        (
            Parser::new_head(),
            DebugHandler::new()
        )
    });
}

#[test]
fn byte_check() {
    // invalid bytes
    loop_non_tokens(b"\r\n\t ", |byte| {
        let (mut p, mut h) = setup!();

        assert_error_byte!(
            p,
            h,
            &[byte],
            Method,
            byte
        );
    });

    // valid lower-cased alphabetical bytes
    loop_tokens(b"Hh", |byte| {
        if byte > 0x60 && byte < 0x7B {
            let (mut p, mut h) = setup!();

            assert_eos!(
                p,
                h,
                &[byte],
                LowerRequestMethod
            );
        }
    });

    // valid upper-cased alphabetical bytes
    loop_tokens(b"Hh", |byte| {
        if !(byte > 0x60 && byte < 0x7B) {
            let (mut p, mut h) = setup!();

            assert_eos!(
                p,
                h,
                &[byte],
                UpperRequestMethod
            );
        }
    });

    for n in &[b'H', b'h'] {
        // valid H|h byte
        let (mut p, mut h) = setup!();

        assert_eos!(
            p,
            h,
            &[*n as u8],
            Detect2
        );
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

    let mut h = CallbackHandler;
    let mut p = Parser::new_head();

    assert_callback!(
        p,
        h,
        b"G",
        UpperRequestMethod
    );
}

#[test]
fn multiple_connect() {
    let (mut p, mut h) = setup!();

    assert_eos!(
        p,
        h,
        b"C",
        UpperRequestMethod
    );

    assert_eq!(
        h.method,
        b"C"
    );

    assert_eos!(
        p,
        h,
        b"O",
        UpperRequestMethod
    );

    assert_eq!(
        h.method,
        b"CO"
    );

    assert_eos!(
        p,
        h,
        b"N",
        UpperRequestMethod
    );

    assert_eq!(
        h.method,
        b"CON"
    );

    assert_eos!(
        p,
        h,
        b"N",
        UpperRequestMethod
    );

    assert_eq!(
        h.method,
        b"CONN"
    );

    assert_eos!(
        p,
        h,
        b"E",
        UpperRequestMethod
    );

    assert_eq!(
        h.method,
        b"CONNE"
    );

    assert_eos!(
        p,
        h,
        b"C",
        UpperRequestMethod
    );

    assert_eq!(
        h.method,
        b"CONNEC"
    );

    assert_eos!(
        p,
        h,
        b"T",
        UpperRequestMethod
    );

    assert_eq!(
        h.method,
        b"CONNECT"
    );

    assert_eos!(
        p,
        h,
        b" ",
        StripRequestUrl
    );

    assert_eq!(
        h.method,
        b"CONNECT"
    );
}

#[test]
fn multiple_delete() {
    let (mut p, mut h) = setup!();

    assert_eos!(
        p,
        h,
        b"D",
        UpperRequestMethod
    );

    assert_eq!(
        h.method,
        b"D"
    );

    assert_eos!(
        p,
        h,
        b"E",
        UpperRequestMethod
    );

    assert_eq!(
        h.method,
        b"DE"
    );

    assert_eos!(
        p,
        h,
        b"L",
        UpperRequestMethod
    );

    assert_eq!(
        h.method,
        b"DEL"
    );

    assert_eos!(
        p,
        h,
        b"E",
        UpperRequestMethod
    );

    assert_eq!(
        h.method,
        b"DELE"
    );

    assert_eos!(
        p,
        h,
        b"T",
        UpperRequestMethod
    );

    assert_eq!(
        h.method,
        b"DELET"
    );

    assert_eos!(
        p,
        h,
        b"E",
        UpperRequestMethod
    );

    assert_eq!(
        h.method,
        b"DELETE"
    );

    assert_eos!(
        p,
        h,
        b" ",
        StripRequestUrl
    );

    assert_eq!(
        h.method,
        b"DELETE"
    );
}

#[test]
fn multiple_get() {
    let (mut p, mut h) = setup!();

    assert_eos!(
        p,
        h,
        b"G",
        UpperRequestMethod
    );

    assert_eq!(
        h.method,
        b"G"
    );

    assert_eos!(
        p,
        h,
        b"E",
        UpperRequestMethod
    );

    assert_eq!(
        h.method,
        b"GE"
    );

    assert_eos!(
        p,
        h,
        b"T",
        UpperRequestMethod
    );

    assert_eq!(
        h.method,
        b"GET"
    );

    assert_eos!(
        p,
        h,
        b" ",
        StripRequestUrl
    );

    assert_eq!(
        h.method,
        b"GET"
    );
}

#[test]
fn multiple_head() {
    let (mut p, mut h) = setup!();

    assert_eos!(
        p,
        h,
        b"H",
        Detect2
    );

    assert_eos!(
        p,
        h,
        b"E",
        UpperRequestMethod
    );

    assert_eq!(
        h.method,
        b"HE"
    );

    assert_eos!(
        p,
        h,
        b"A",
        UpperRequestMethod
    );

    assert_eq!(
        h.method,
        b"HEA"
    );

    assert_eos!(
        p,
        h,
        b"D",
        UpperRequestMethod
    );

    assert_eq!(
        h.method,
        b"HEAD"
    );

    assert_eos!(
        p,
        h,
        b" ",
        StripRequestUrl
    );

    assert_eq!(
        h.method,
        b"HEAD"
    );
}

#[test]
fn multiple_options() {
    let (mut p, mut h) = setup!();

    assert_eos!(
        p,
        h,
        b"O",
        UpperRequestMethod
    );

    assert_eq!(
        h.method,
        b"O"
    );

    assert_eos!(
        p,
        h,
        b"P",
        UpperRequestMethod
    );

    assert_eq!(
        h.method,
        b"OP"
    );

    assert_eos!(
        p,
        h,
        b"T",
        UpperRequestMethod
    );

    assert_eq!(
        h.method,
        b"OPT"
    );

    assert_eos!(
        p,
        h,
        b"I",
        UpperRequestMethod
    );

    assert_eq!(
        h.method,
        b"OPTI"
    );

    assert_eos!(
        p,
        h,
        b"O",
        UpperRequestMethod
    );

    assert_eq!(
        h.method,
        b"OPTIO"
    );

    assert_eos!(
        p,
        h,
        b"N",
        UpperRequestMethod
    );

    assert_eq!(
        h.method,
        b"OPTION"
    );

    assert_eos!(
        p,
        h,
        b"S",
        UpperRequestMethod
    );

    assert_eq!(
        h.method,
        b"OPTIONS"
    );

    assert_eos!(
        p,
        h,
        b" ",
        StripRequestUrl
    );

    assert_eq!(
        h.method,
        b"OPTIONS"
    );
}

#[test]
fn multiple_post() {
    let (mut p, mut h) = setup!();

    assert_eos!(
        p,
        h,
        b"P",
        UpperRequestMethod
    );

    assert_eq!(
        h.method,
        b"P"
    );

    assert_eos!(
        p,
        h,
        b"O",
        UpperRequestMethod
    );

    assert_eq!(
        h.method,
        b"PO"
    );

    assert_eos!(
        p,
        h,
        b"S",
        UpperRequestMethod
    );

    assert_eq!(
        h.method,
        b"POS"
    );

    assert_eos!(
        p,
        h,
        b"T",
        UpperRequestMethod
    );

    assert_eq!(
        h.method,
        b"POST"
    );

    assert_eos!(
        p,
        h,
        b" ",
        StripRequestUrl
    );

    assert_eq!(
        h.method,
        b"POST"
    );
}

#[test]
fn multiple_put() {
    let (mut p, mut h) = setup!();

    assert_eos!(
        p,
        h,
        b"P",
        UpperRequestMethod
    );

    assert_eq!(
        h.method,
        b"P"
    );

    assert_eos!(
        p,
        h,
        b"U",
        UpperRequestMethod
    );

    assert_eq!(
        h.method,
        b"PU"
    );

    assert_eos!(
        p,
        h,
        b"T",
        UpperRequestMethod
    );

    assert_eq!(
        h.method,
        b"PUT"
    );

    assert_eos!(
        p,
        h,
        b" ",
        StripRequestUrl
    );

    assert_eq!(
        h.method,
        b"PUT"
    );
}

#[test]
fn multiple_trace() {
    let (mut p, mut h) = setup!();

    assert_eos!(
        p,
        h,
        b"T",
        UpperRequestMethod
    );

    assert_eq!(
        h.method,
        b"T"
    );

    assert_eos!(
        p,
        h,
        b"R",
        UpperRequestMethod
    );

    assert_eq!(
        h.method,
        b"TR"
    );

    assert_eos!(
        p,
        h,
        b"A",
        UpperRequestMethod
    );

    assert_eq!(
        h.method,
        b"TRA"
    );

    assert_eos!(
        p,
        h,
        b"C",
        UpperRequestMethod
    );

    assert_eq!(
        h.method,
        b"TRAC"
    );

    assert_eos!(
        p,
        h,
        b"E",
        UpperRequestMethod
    );

    assert_eq!(
        h.method,
        b"TRACE"
    );

    assert_eos!(
        p,
        h,
        b" ",
        StripRequestUrl
    );

    assert_eq!(
        h.method,
        b"TRACE"
    );
}

#[test]
fn multiple_unknown() {
    let (mut p, mut h) = setup!();

    assert_eos!(
        p,
        h,
        b"U",
        UpperRequestMethod
    );

    assert_eq!(
        h.method,
        b"U"
    );

    assert_eos!(
        p,
        h,
        b"N",
        UpperRequestMethod
    );

    assert_eq!(
        h.method,
        b"UN"
    );

    assert_eos!(
        p,
        h,
        b"K",
        UpperRequestMethod
    );

    assert_eq!(
        h.method,
        b"UNK"
    );

    assert_eos!(
        p,
        h,
        b"N",
        UpperRequestMethod
    );

    assert_eq!(
        h.method,
        b"UNKN"
    );

    assert_eos!(
        p,
        h,
        b"O",
        UpperRequestMethod
    );

    assert_eq!(
        h.method,
        b"UNKNO"
    );

    assert_eos!(
        p,
        h,
        b"W",
        UpperRequestMethod
    );

    assert_eq!(
        h.method,
        b"UNKNOW"
    );

    assert_eos!(
        p,
        h,
        b"N",
        UpperRequestMethod
    );

    assert_eq!(
        h.method,
        b"UNKNOWN"
    );

    assert_eos!(
        p,
        h,
        b" ",
        StripRequestUrl
    );

    assert_eq!(
        h.method,
        b"UNKNOWN"
    );
}

#[test]
fn normalize() {
    let (mut p, mut h) = setup!();

    assert_eos!(
        p,
        h,
        b"g",
        LowerRequestMethod
    );

    assert_eq!(
        h.method,
        b"G"
    );

    assert_eos!(
        p,
        h,
        b"E",
        UpperRequestMethod
    );

    assert_eq!(
        h.method,
        b"GE"
    );

    assert_eos!(
        p,
        h,
        b"t",
        LowerRequestMethod
    );

    assert_eq!(
        h.method,
        b"GET"
    );

    assert_eos!(
        p,
        h,
        b" ",
        StripRequestUrl
    );

    assert_eq!(
        h.method,
        b"GET"
    );
}

#[test]
fn single_connect() {
    let (mut p, mut h) = setup!();

    assert_eos!(
        p,
        h,
        b"CONNECT ",
        StripRequestUrl
    );

    assert_eq!(
        h.method,
        b"CONNECT"
    );
}

#[test]
fn single_delete() {
    let (mut p, mut h) = setup!();

    assert_eos!(
        p,
        h,
        b"DELETE  ",
        StripRequestUrl
    );

    assert_eq!(
        h.method,
        b"DELETE"
    );
}

#[test]
fn single_get() {
    let (mut p, mut h) = setup!();

    assert_eos!(
        p,
        h,
        b"GET     ",
        StripRequestUrl
    );

    assert_eq!(
        h.method,
        b"GET"
    );
}

#[test]
fn single_head() {
    let (mut p, mut h) = setup!();

    assert_eos!(
        p,
        h,
        b"HEAD    ",
        StripRequestUrl
    );

    assert_eq!(
        h.method,
        b"HEAD"
    );
}

#[test]
fn single_options() {
    let (mut p, mut h) = setup!();

    assert_eos!(
        p,
        h,
        b"OPTIONS ",
        StripRequestUrl
    );

    assert_eq!(
        h.method,
        b"OPTIONS"
    );
}

#[test]
fn single_post() {
    let (mut p, mut h) = setup!();

    assert_eos!(
        p,
        h,
        b"POST    ",
        StripRequestUrl
    );

    assert_eq!(
        h.method,
        b"POST"
    );
}

#[test]
fn single_put() {
    let (mut p, mut h) = setup!();

    assert_eos!(
        p,
        h,
        b"PUT     ",
        StripRequestUrl
    );

    assert_eq!(
        h.method,
        b"PUT"
    );
}

#[test]
fn single_trace() {
    let (mut p, mut h) = setup!();

    assert_eos!(
        p,
        h,
        b"TRACE   ",
        StripRequestUrl
    );

    assert_eq!(
        h.method,
        b"TRACE"
    );
}

#[test]
fn single_unknown() {
    let (mut p, mut h) = setup!();

    assert_eos!(
        p,
        h,
        b"UNKNOWN ",
        StripRequestUrl
    );

    assert_eq!(
        h.method,
        b"UNKNOWN"
    );
}

#[test]
fn starting_space() {
    let (mut p, mut h) = setup!();

    assert_eos!(
        p,
        h,
        b"   ",
        StripDetect
    );
}
