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
    });
}

#[test]
fn byte_check() {
    // invalid bytes
    loop_non_tokens(b"\r\n\t ", |byte| {
        let mut h = DebugHandler::new();
        let mut p = Parser::new();

        setup!(p, h);

        assert_error_byte!(p, h,
                           &[byte],
                           ParserError::Method,
                           byte);
    });

    // valid bytes
    loop_tokens(b"Hh", |byte| {
        let mut h = DebugHandler::new();
        let mut p = Parser::new();

        setup!(p, h);

        assert_eos!(p, h,
                    &[byte],
                    ParserState::RequestMethod);
    });

    for n in &[b'H', b'h'] {
        // valid H|h byte
        let mut h = DebugHandler::new();
        let mut p = Parser::new();

        setup!(p, h);

        assert_eos!(p, h,
                    &[*n as u8],
                    ParserState::Detect2);
    }
}

#[test]
fn callback_exit() {
    struct X;

    impl HttpHandler for X {
        fn on_method(&mut self, _method: &[u8]) -> bool {
            false
        }
    }

    let mut h = X{};
    let mut p = Parser::new();

    setup!(p, h);

    assert_callback!(p, h,
                     b"G",
                     ParserState::RequestMethod);
}

#[test]
fn multiple_connect() {
    let mut h = DebugHandler::new();
    let mut p = Parser::new();

    setup!(p, h);

    assert_eos!(p, h,
               b"C",
               ParserState::RequestMethod);
    assert_eq!(h.method, b"C");

    assert_eos!(p, h,
               b"O",
               ParserState::RequestMethod);
    assert_eq!(h.method, b"CO");

    assert_eos!(p, h,
               b"N",
               ParserState::RequestMethod);
    assert_eq!(h.method, b"CON");

    assert_eos!(p, h,
               b"N",
               ParserState::RequestMethod);
    assert_eq!(h.method, b"CONN");

    assert_eos!(p, h,
               b"E",
               ParserState::RequestMethod);
    assert_eq!(h.method, b"CONNE");

    assert_eos!(p, h,
               b"C",
               ParserState::RequestMethod);
    assert_eq!(h.method, b"CONNEC");

    assert_eos!(p, h,
               b"T",
               ParserState::RequestMethod);
    assert_eq!(h.method, b"CONNECT");

    assert_eos!(p, h,
               b" ",
               ParserState::StripRequestUrl);
    assert_eq!(h.method, b"CONNECT");
}

#[test]
fn multiple_delete() {
    let mut h = DebugHandler::new();
    let mut p = Parser::new();

    setup!(p, h);

    assert_eos!(p, h,
                b"D",
                ParserState::RequestMethod);
    assert_eq!(h.method, b"D");

    assert_eos!(p, h,
                b"E",
                ParserState::RequestMethod);
    assert_eq!(h.method, b"DE");

    assert_eos!(p, h,
                b"L",
                ParserState::RequestMethod);
    assert_eq!(h.method, b"DEL");

    assert_eos!(p, h,
                b"E",
                ParserState::RequestMethod);
    assert_eq!(h.method, b"DELE");

    assert_eos!(p, h,
                b"T",
                ParserState::RequestMethod);
    assert_eq!(h.method, b"DELET");

    assert_eos!(p, h,
                b"E",
                ParserState::RequestMethod);
    assert_eq!(h.method, b"DELETE");

    assert_eos!(p, h,
                b" ",
                ParserState::StripRequestUrl);
    assert_eq!(h.method, b"DELETE");
}

#[test]
fn multiple_get() {
    let mut h = DebugHandler::new();
    let mut p = Parser::new();

    setup!(p, h);

    assert_eos!(p, h,
                b"G",
                ParserState::RequestMethod);
    assert_eq!(h.method, b"G");

    assert_eos!(p, h,
                b"E",
                ParserState::RequestMethod);
    assert_eq!(h.method, b"GE");

    assert_eos!(p, h,
                b"T",
                ParserState::RequestMethod);
    assert_eq!(h.method, b"GET");

    assert_eos!(p, h,
                b" ",
                ParserState::StripRequestUrl);
    assert_eq!(h.method, b"GET");
}

#[test]
fn multiple_head() {
    let mut h = DebugHandler::new();
    let mut p = Parser::new();

    setup!(p, h);

    assert_eos!(p, h,
                b"H",
                ParserState::Detect2);
    assert_eos!(p, h,
                b"E",
                ParserState::RequestMethod);
    assert_eq!(h.method, b"HE");

    assert_eos!(p, h,
                b"A",
                ParserState::RequestMethod);
    assert_eq!(h.method, b"HEA");

    assert_eos!(p, h,
                b"D",
                ParserState::RequestMethod);
    assert_eq!(h.method, b"HEAD");

    assert_eos!(p, h,
                b" ",
                ParserState::StripRequestUrl);
    assert_eq!(h.method, b"HEAD");
}

#[test]
fn multiple_options() {
    let mut h = DebugHandler::new();
    let mut p = Parser::new();

    setup!(p, h);

    assert_eos!(p, h,
                b"O",
                ParserState::RequestMethod);
    assert_eq!(h.method, b"O");

    assert_eos!(p, h,
                b"P",
                ParserState::RequestMethod);
    assert_eq!(h.method, b"OP");

    assert_eos!(p, h,
                b"T",
                ParserState::RequestMethod);
    assert_eq!(h.method, b"OPT");

    assert_eos!(p, h,
                b"I",
                ParserState::RequestMethod);
    assert_eq!(h.method, b"OPTI");

    assert_eos!(p, h,
                b"O",
                ParserState::RequestMethod);
    assert_eq!(h.method, b"OPTIO");

    assert_eos!(p, h,
                b"N",
                ParserState::RequestMethod);
    assert_eq!(h.method, b"OPTION");

    assert_eos!(p, h,
                b"S",
                ParserState::RequestMethod);
    assert_eq!(h.method, b"OPTIONS");

    assert_eos!(p, h,
                b" ",
                ParserState::StripRequestUrl);
    assert_eq!(h.method, b"OPTIONS");
}

#[test]
fn multiple_post() {
    let mut h = DebugHandler::new();
    let mut p = Parser::new();

    setup!(p, h);

    assert_eos!(p, h,
                b"P",
                ParserState::RequestMethod);
    assert_eq!(h.method, b"P");

    assert_eos!(p, h,
                b"O",
                ParserState::RequestMethod);
    assert_eq!(h.method, b"PO");

    assert_eos!(p, h,
                b"S",
                ParserState::RequestMethod);
    assert_eq!(h.method, b"POS");

    assert_eos!(p, h,
                b"T",
                ParserState::RequestMethod);
    assert_eq!(h.method, b"POST");

    assert_eos!(p, h,
                b" ",
                ParserState::StripRequestUrl);
    assert_eq!(h.method, b"POST");
}

#[test]
fn multiple_put() {
    let mut h = DebugHandler::new();
    let mut p = Parser::new();

    setup!(p, h);

    assert_eos!(p, h,
                b"P",
                ParserState::RequestMethod);
    assert_eq!(h.method, b"P");

    assert_eos!(p, h,
                b"U",
                ParserState::RequestMethod);
    assert_eq!(h.method, b"PU");

    assert_eos!(p, h,
                b"T",
                ParserState::RequestMethod);
    assert_eq!(h.method, b"PUT");

    assert_eos!(p, h,
                b" ",
                ParserState::StripRequestUrl);
    assert_eq!(h.method, b"PUT");
}

#[test]
fn multiple_trace() {
    let mut h = DebugHandler::new();
    let mut p = Parser::new();

    setup!(p, h);

    assert_eos!(p, h,
                b"T",
                ParserState::RequestMethod);
    assert_eq!(h.method, b"T");

    assert_eos!(p, h,
                b"R",
                ParserState::RequestMethod);
    assert_eq!(h.method, b"TR");

    assert_eos!(p, h,
                b"A",
                ParserState::RequestMethod);
    assert_eq!(h.method, b"TRA");

    assert_eos!(p, h,
                b"C",
                ParserState::RequestMethod);
    assert_eq!(h.method, b"TRAC");

    assert_eos!(p, h,
                b"E",
                ParserState::RequestMethod);
    assert_eq!(h.method, b"TRACE");

    assert_eos!(p, h,
                b" ",
                ParserState::StripRequestUrl);
    assert_eq!(h.method, b"TRACE");
}

#[test]
fn multiple_unknown() {
    let mut h = DebugHandler::new();
    let mut p = Parser::new();

    setup!(p, h);

    assert_eos!(p, h,
                b"U",
                ParserState::RequestMethod);
    assert_eq!(h.method, b"U");

    assert_eos!(p, h,
                b"N",
                ParserState::RequestMethod);
    assert_eq!(h.method, b"UN");

    assert_eos!(p, h,
                b"K",
                ParserState::RequestMethod);
    assert_eq!(h.method, b"UNK");

    assert_eos!(p, h,
                b"N",
                ParserState::RequestMethod);
    assert_eq!(h.method, b"UNKN");

    assert_eos!(p, h,
                b"O",
                ParserState::RequestMethod);
    assert_eq!(h.method, b"UNKNO");

    assert_eos!(p, h,
                b"W",
                ParserState::RequestMethod);
    assert_eq!(h.method, b"UNKNOW");

    assert_eos!(p, h,
                b"N",
                ParserState::RequestMethod);
    assert_eq!(h.method, b"UNKNOWN");

    assert_eos!(p, h,
                b" ",
                ParserState::StripRequestUrl);
    assert_eq!(h.method, b"UNKNOWN");
}

#[test]
fn single_connect() {
    let mut h = DebugHandler::new();
    let mut p = Parser::new();

    setup!(p, h);

    assert_eos!(p, h,
                b"CONNECT ",
                ParserState::StripRequestUrl);
    assert_eq!(h.method, b"CONNECT");
}

#[test]
fn single_delete() {
    let mut h = DebugHandler::new();
    let mut p = Parser::new();

    setup!(p, h);

    assert_eos!(p, h,
                b"DELETE  ",
                ParserState::StripRequestUrl);
    assert_eq!(h.method, b"DELETE");
}

#[test]
fn single_get() {
    let mut h = DebugHandler::new();
    let mut p = Parser::new();

    setup!(p, h);

    assert_eos!(p, h,
                b"GET     ",
                ParserState::StripRequestUrl);
    assert_eq!(h.method, b"GET");
}

#[test]
fn single_head() {
    let mut h = DebugHandler::new();
    let mut p = Parser::new();

    setup!(p, h);

    assert_eos!(p, h,
                b"HEAD    ",
                ParserState::StripRequestUrl);
    assert_eq!(h.method, b"HEAD");
}

#[test]
fn single_options() {
    let mut h = DebugHandler::new();
    let mut p = Parser::new();

    setup!(p, h);

    assert_eos!(p, h,
                b"OPTIONS ",
                ParserState::StripRequestUrl);
    assert_eq!(h.method, b"OPTIONS");
}

#[test]
fn single_post() {
    let mut h = DebugHandler::new();
    let mut p = Parser::new();

    setup!(p, h);

    assert_eos!(p, h,
                b"POST    ",
                ParserState::StripRequestUrl);
    assert_eq!(h.method, b"POST");
}

#[test]
fn single_put() {
    let mut h = DebugHandler::new();
    let mut p = Parser::new();

    setup!(p, h);

    assert_eos!(p, h,
                b"PUT     ",
                ParserState::StripRequestUrl);
    assert_eq!(h.method, b"PUT");
}

#[test]
fn single_trace() {
    let mut h = DebugHandler::new();
    let mut p = Parser::new();

    setup!(p, h);

    assert_eos!(p, h,
                b"TRACE   ",
                ParserState::StripRequestUrl);
    assert_eq!(h.method, b"TRACE");
}

#[test]
fn single_unknown() {
    let mut h = DebugHandler::new();
    let mut p = Parser::new();

    setup!(p, h);

    assert_eos!(p, h,
                b"UNKNOWN ",
                ParserState::StripRequestUrl);
    assert_eq!(h.method, b"UNKNOWN");
}

#[test]
fn starting_space() {
    let mut h = DebugHandler::new();
    let mut p = Parser::new();

    setup!(p, h);

    assert_eos!(p, h,
                b"   ",
                ParserState::StripDetect);
}
