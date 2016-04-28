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
use test::{ loop_non_tokens,
            loop_tokens };
use url::*;
use std::str;

#[test]
fn byte_check() {
    // invalid bytes
    loop_non_tokens(b" ", |byte| {
        let mut h = DebugHandler::new();
        let mut p = Parser::new(StreamType::Request);

        assert!(match p.parse(&mut h, b"a") {
            Ok(Success::Eof(1)) => {
                assert_eq!(p.get_state(), State::RequestMethod);
                true
            },
            _ => false
        });

        assert!(match p.parse(&mut h, &[byte]) {
            Err(ParserError::Method(_,x)) => {
                assert_eq!(x, byte);
                assert_eq!(p.get_state(), State::Dead);
                true
            },
            _ => false
        });
    });

    // valid bytes
    loop_tokens(b"", |byte| {
        let mut h = DebugHandler::new();
        let mut p = Parser::new(StreamType::Request);

        assert!(match p.parse(&mut h, &[byte]) {
            Ok(Success::Eof(1)) => {
                assert_eq!(p.get_state(), State::RequestMethod);
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
        fn on_method(&mut self, _data: &[u8]) -> bool {
            false
        }
    }

    impl ParamHandler for X {}

    let mut h = X{};
    let mut p = Parser::new(StreamType::Request);

    assert!(match p.parse(&mut h, b"G") {
        Ok(Success::Callback(1)) => true,
        _ => false
    });
}

#[allow(cyclomatic_complexity)]
#[test]
fn multiple_pieces_connect() {
    let mut h = DebugHandler::new();
    let mut p = Parser::new(StreamType::Request);

    assert!(match p.parse(&mut h, b"C") {
        Ok(Success::Eof(1)) => {
            assert_eq!(h.method, b"C");
            assert_eq!(p.get_state(), State::RequestMethod);
            true
        },
        _ => false
    });

    assert!(match p.parse(&mut h, b"O") {
        Ok(Success::Eof(1)) => {
            assert_eq!(h.method, b"CO");
            assert_eq!(p.get_state(), State::RequestMethod);
            true
        },
        _ => false
    });

    assert!(match p.parse(&mut h, b"N") {
        Ok(Success::Eof(1)) => {
            assert_eq!(h.method, b"CON");
            assert_eq!(p.get_state(), State::RequestMethod);
            true
        },
        _ => false
    });

    assert!(match p.parse(&mut h, b"N") {
        Ok(Success::Eof(1)) => {
            assert_eq!(h.method, b"CONN");
            assert_eq!(p.get_state(), State::RequestMethod);
            true
        },
        _ => false
    });

    assert!(match p.parse(&mut h, b"E") {
        Ok(Success::Eof(1)) => {
            assert_eq!(h.method, b"CONNE");
            assert_eq!(p.get_state(), State::RequestMethod);
            true
        },
        _ => false
    });

    assert!(match p.parse(&mut h, b"C") {
        Ok(Success::Eof(1)) => {
            assert_eq!(h.method, b"CONNEC");
            assert_eq!(p.get_state(), State::RequestMethod);
            true
        },
        _ => false
    });

    assert!(match p.parse(&mut h, b"T") {
        Ok(Success::Eof(1)) => {
            assert_eq!(h.method, b"CONNECT");
            assert_eq!(p.get_state(), State::RequestMethod);
            true
        },
        _ => false
    });

    assert!(match p.parse(&mut h, b" ") {
        Ok(Success::Eof(1)) => {
            assert_eq!(h.method, b"CONNECT");
            assert_eq!(p.get_state(), State::StripRequestUrl);
            true
        },
        _ => false
    });
}

#[allow(cyclomatic_complexity)]
#[test]
fn multiple_pieces_delete() {
    let mut h = DebugHandler::new();
    let mut p = Parser::new(StreamType::Request);

    assert!(match p.parse(&mut h, b"D") {
        Ok(Success::Eof(1)) => {
            assert_eq!(h.method, b"D");
            assert_eq!(p.get_state(), State::RequestMethod);
            true
        },
        _ => false
    });

    assert!(match p.parse(&mut h, b"E") {
        Ok(Success::Eof(1)) => {
            assert_eq!(h.method, b"DE");
            assert_eq!(p.get_state(), State::RequestMethod);
            true
        },
        _ => false
    });

    assert!(match p.parse(&mut h, b"L") {
        Ok(Success::Eof(1)) => {
            assert_eq!(h.method, b"DEL");
            assert_eq!(p.get_state(), State::RequestMethod);
            true
        },
        _ => false
    });

    assert!(match p.parse(&mut h, b"E") {
        Ok(Success::Eof(1)) => {
            assert_eq!(h.method, b"DELE");
            assert_eq!(p.get_state(), State::RequestMethod);
            true
        },
        _ => false
    });

    assert!(match p.parse(&mut h, b"T") {
        Ok(Success::Eof(1)) => {
            assert_eq!(h.method, b"DELET");
            assert_eq!(p.get_state(), State::RequestMethod);
            true
        },
        _ => false
    });

    assert!(match p.parse(&mut h, b"E") {
        Ok(Success::Eof(1)) => {
            assert_eq!(h.method, b"DELETE");
            assert_eq!(p.get_state(), State::RequestMethod);
            true
        },
        _ => false
    });

    assert!(match p.parse(&mut h, b" ") {
        Ok(Success::Eof(1)) => {
            assert_eq!(h.method, b"DELETE");
            assert_eq!(p.get_state(), State::StripRequestUrl);
            true
        },
        _ => false
    });
}

#[allow(cyclomatic_complexity)]
#[test]
fn multiple_pieces_get() {
    let mut h = DebugHandler::new();
    let mut p = Parser::new(StreamType::Request);

    assert!(match p.parse(&mut h, b"G") {
        Ok(Success::Eof(1)) => {
            assert_eq!(h.method, b"G");
            assert_eq!(p.get_state(), State::RequestMethod);
            true
        },
        _ => false
    });

    assert!(match p.parse(&mut h, b"E") {
        Ok(Success::Eof(1)) => {
            assert_eq!(h.method, b"GE");
            assert_eq!(p.get_state(), State::RequestMethod);
            true
        },
        _ => false
    });

    assert!(match p.parse(&mut h, b"T") {
        Ok(Success::Eof(1)) => {
            assert_eq!(h.method, b"GET");
            assert_eq!(p.get_state(), State::RequestMethod);
            true
        },
        _ => false
    });

    assert!(match p.parse(&mut h, b" ") {
        Ok(Success::Eof(1)) => {
            assert_eq!(h.method, b"GET");
            assert_eq!(p.get_state(), State::StripRequestUrl);
            true
        },
        _ => false
    });
}

#[allow(cyclomatic_complexity)]
#[test]
fn multiple_pieces_head() {
    let mut h = DebugHandler::new();
    let mut p = Parser::new(StreamType::Request);

    assert!(match p.parse(&mut h, b"H") {
        Ok(Success::Eof(1)) => {
            assert_eq!(h.method, b"H");
            assert_eq!(p.get_state(), State::RequestMethod);
            true
        },
        _ => false
    });

    assert!(match p.parse(&mut h, b"E") {
        Ok(Success::Eof(1)) => {
            assert_eq!(h.method, b"HE");
            assert_eq!(p.get_state(), State::RequestMethod);
            true
        },
        _ => false
    });

    assert!(match p.parse(&mut h, b"A") {
        Ok(Success::Eof(1)) => {
            assert_eq!(h.method, b"HEA");
            assert_eq!(p.get_state(), State::RequestMethod);
            true
        },
        _ => false
    });

    assert!(match p.parse(&mut h, b"D") {
        Ok(Success::Eof(1)) => {
            assert_eq!(h.method, b"HEAD");
            assert_eq!(p.get_state(), State::RequestMethod);
            true
        },
        _ => false
    });

    assert!(match p.parse(&mut h, b" ") {
        Ok(Success::Eof(1)) => {
            assert_eq!(h.method, b"HEAD");
            assert_eq!(p.get_state(), State::StripRequestUrl);
            true
        },
        _ => false
    });
}

#[allow(cyclomatic_complexity)]
#[test]
fn multiple_pieces_options() {
    let mut h = DebugHandler::new();
    let mut p = Parser::new(StreamType::Request);

    assert!(match p.parse(&mut h, b"O") {
        Ok(Success::Eof(1)) => {
            assert_eq!(h.method, b"O");
            assert_eq!(p.get_state(), State::RequestMethod);
            true
        },
        _ => false
    });

    assert!(match p.parse(&mut h, b"P") {
        Ok(Success::Eof(1)) => {
            assert_eq!(h.method, b"OP");
            assert_eq!(p.get_state(), State::RequestMethod);
            true
        },
        _ => false
    });

    assert!(match p.parse(&mut h, b"T") {
        Ok(Success::Eof(1)) => {
            assert_eq!(h.method, b"OPT");
            assert_eq!(p.get_state(), State::RequestMethod);
            true
        },
        _ => false
    });

    assert!(match p.parse(&mut h, b"I") {
        Ok(Success::Eof(1)) => {
            assert_eq!(h.method, b"OPTI");
            assert_eq!(p.get_state(), State::RequestMethod);
            true
        },
        _ => false
    });

    assert!(match p.parse(&mut h, b"O") {
        Ok(Success::Eof(1)) => {
            assert_eq!(h.method, b"OPTIO");
            assert_eq!(p.get_state(), State::RequestMethod);
            true
        },
        _ => false
    });

    assert!(match p.parse(&mut h, b"N") {
        Ok(Success::Eof(1)) => {
            assert_eq!(h.method, b"OPTION");
            assert_eq!(p.get_state(), State::RequestMethod);
            true
        },
        _ => false
    });

    assert!(match p.parse(&mut h, b"S") {
        Ok(Success::Eof(1)) => {
            assert_eq!(h.method, b"OPTIONS");
            assert_eq!(p.get_state(), State::RequestMethod);
            true
        },
        _ => false
    });

    assert!(match p.parse(&mut h, b" ") {
        Ok(Success::Eof(1)) => {
            assert_eq!(h.method, b"OPTIONS");
            assert_eq!(p.get_state(), State::StripRequestUrl);
            true
        },
        _ => false
    });
}

#[allow(cyclomatic_complexity)]
#[test]
fn multiple_pieces_post() {
    let mut h = DebugHandler::new();
    let mut p = Parser::new(StreamType::Request);

    assert!(match p.parse(&mut h, b"P") {
        Ok(Success::Eof(1)) => {
            assert_eq!(h.method, b"P");
            assert_eq!(p.get_state(), State::RequestMethod);
            true
        },
        _ => false
    });

    assert!(match p.parse(&mut h, b"O") {
        Ok(Success::Eof(1)) => {
            assert_eq!(h.method, b"PO");
            assert_eq!(p.get_state(), State::RequestMethod);
            true
        },
        _ => false
    });

    assert!(match p.parse(&mut h, b"S") {
        Ok(Success::Eof(1)) => {
            assert_eq!(h.method, b"POS");
            assert_eq!(p.get_state(), State::RequestMethod);
            true
        },
        _ => false
    });

    assert!(match p.parse(&mut h, b"T") {
        Ok(Success::Eof(1)) => {
            assert_eq!(h.method, b"POST");
            assert_eq!(p.get_state(), State::RequestMethod);
            true
        },
        _ => false
    });

    assert!(match p.parse(&mut h, b" ") {
        Ok(Success::Eof(1)) => {
            assert_eq!(h.method, b"POST");
            assert_eq!(p.get_state(), State::StripRequestUrl);
            true
        },
        _ => false
    });
}

#[allow(cyclomatic_complexity)]
#[test]
fn multiple_pieces_put() {
    let mut h = DebugHandler::new();
    let mut p = Parser::new(StreamType::Request);

    assert!(match p.parse(&mut h, b"P") {
        Ok(Success::Eof(1)) => {
            assert_eq!(h.method, b"P");
            assert_eq!(p.get_state(), State::RequestMethod);
            true
        },
        _ => false
    });

    assert!(match p.parse(&mut h, b"U") {
        Ok(Success::Eof(1)) => {
            assert_eq!(h.method, b"PU");
            assert_eq!(p.get_state(), State::RequestMethod);
            true
        },
        _ => false
    });

    assert!(match p.parse(&mut h, b"T") {
        Ok(Success::Eof(1)) => {
            assert_eq!(h.method, b"PUT");
            assert_eq!(p.get_state(), State::RequestMethod);
            true
        },
        _ => false
    });

    assert!(match p.parse(&mut h, b" ") {
        Ok(Success::Eof(1)) => {
            assert_eq!(h.method, b"PUT");
            assert_eq!(p.get_state(), State::StripRequestUrl);
            true
        },
        _ => false
    });
}

#[allow(cyclomatic_complexity)]
#[test]
fn multiple_pieces_trace() {
    let mut h = DebugHandler::new();
    let mut p = Parser::new(StreamType::Request);

    assert!(match p.parse(&mut h, b"T") {
        Ok(Success::Eof(1)) => {
            assert_eq!(h.method, b"T");
            assert_eq!(p.get_state(), State::RequestMethod);
            true
        },
        _ => false
    });

    assert!(match p.parse(&mut h, b"R") {
        Ok(Success::Eof(1)) => {
            assert_eq!(h.method, b"TR");
            assert_eq!(p.get_state(), State::RequestMethod);
            true
        },
        _ => false
    });

    assert!(match p.parse(&mut h, b"A") {
        Ok(Success::Eof(1)) => {
            assert_eq!(h.method, b"TRA");
            assert_eq!(p.get_state(), State::RequestMethod);
            true
        },
        _ => false
    });

    assert!(match p.parse(&mut h, b"C") {
        Ok(Success::Eof(1)) => {
            assert_eq!(h.method, b"TRAC");
            assert_eq!(p.get_state(), State::RequestMethod);
            true
        },
        _ => false
    });

    assert!(match p.parse(&mut h, b"E") {
        Ok(Success::Eof(1)) => {
            assert_eq!(h.method, b"TRACE");
            assert_eq!(p.get_state(), State::RequestMethod);
            true
        },
        _ => false
    });

    assert!(match p.parse(&mut h, b" ") {
        Ok(Success::Eof(1)) => {
            assert_eq!(h.method, b"TRACE");
            assert_eq!(p.get_state(), State::StripRequestUrl);
            true
        },
        _ => false
    });
}

#[allow(cyclomatic_complexity)]
#[test]
fn multiple_pieces_unknown() {
    let mut h = DebugHandler::new();
    let mut p = Parser::new(StreamType::Request);

    assert!(match p.parse(&mut h, b"U") {
        Ok(Success::Eof(1)) => {
            assert_eq!(h.method, b"U");
            assert_eq!(p.get_state(), State::RequestMethod);
            true
        },
        _ => false
    });

    assert!(match p.parse(&mut h, b"N") {
        Ok(Success::Eof(1)) => {
            assert_eq!(h.method, b"UN");
            assert_eq!(p.get_state(), State::RequestMethod);
            true
        },
        _ => false
    });

    assert!(match p.parse(&mut h, b"K") {
        Ok(Success::Eof(1)) => {
            assert_eq!(h.method, b"UNK");
            assert_eq!(p.get_state(), State::RequestMethod);
            true
        },
        _ => false
    });

    assert!(match p.parse(&mut h, b"N") {
        Ok(Success::Eof(1)) => {
            assert_eq!(h.method, b"UNKN");
            assert_eq!(p.get_state(), State::RequestMethod);
            true
        },
        _ => false
    });

    assert!(match p.parse(&mut h, b"O") {
        Ok(Success::Eof(1)) => {
            assert_eq!(h.method, b"UNKNO");
            assert_eq!(p.get_state(), State::RequestMethod);
            true
        },
        _ => false
    });

    assert!(match p.parse(&mut h, b"W") {
        Ok(Success::Eof(1)) => {
            assert_eq!(h.method, b"UNKNOW");
            assert_eq!(p.get_state(), State::RequestMethod);
            true
        },
        _ => false
    });

    assert!(match p.parse(&mut h, b"N") {
        Ok(Success::Eof(1)) => {
            assert_eq!(h.method, b"UNKNOWN");
            assert_eq!(p.get_state(), State::RequestMethod);
            true
        },
        _ => false
    });

    assert!(match p.parse(&mut h, b" ") {
        Ok(Success::Eof(1)) => {
            assert_eq!(h.method, b"UNKNOWN");
            assert_eq!(p.get_state(), State::StripRequestUrl);
            true
        },
        _ => false
    });
}

#[test]
fn one_piece_connect() {
    let mut h = DebugHandler::new();
    let mut p = Parser::new(StreamType::Request);

    assert!(match p.parse(&mut h, b"CONNECT ") {
        Ok(Success::Eof(8)) => {
            assert_eq!(h.method, b"CONNECT");
            assert_eq!(p.get_state(), State::StripRequestUrl);
            true
        },
        _ => false
    });
}

#[test]
fn one_piece_delete() {
    let mut h = DebugHandler::new();
    let mut p = Parser::new(StreamType::Request);

    assert!(match p.parse(&mut h, b"DELETE ") {
        Ok(Success::Eof(7)) => {
            assert_eq!(h.method, b"DELETE");
            assert_eq!(p.get_state(), State::StripRequestUrl);
            true
        },
        _ => false
    });
}

#[test]
fn one_piece_get() {
    let mut h = DebugHandler::new();
    let mut p = Parser::new(StreamType::Request);

    assert!(match p.parse(&mut h, b"GET ") {
        Ok(Success::Eof(4)) => {
            assert_eq!(h.method, b"GET");
            assert_eq!(p.get_state(), State::StripRequestUrl);
            true
        },
        _ => false
    });
}

#[test]
fn one_piece_head() {
    let mut h = DebugHandler::new();
    let mut p = Parser::new(StreamType::Request);

    assert!(match p.parse(&mut h, b"HEAD ") {
        Ok(Success::Eof(5)) => {
            assert_eq!(h.method, b"HEAD");
            assert_eq!(p.get_state(), State::StripRequestUrl);
            true
        },
        _ => false
    });
}

#[test]
fn one_piece_options() {
    let mut h = DebugHandler::new();
    let mut p = Parser::new(StreamType::Request);

    assert!(match p.parse(&mut h, b"OPTIONS ") {
        Ok(Success::Eof(8)) => {
            assert_eq!(h.method, b"OPTIONS");
            assert_eq!(p.get_state(), State::StripRequestUrl);
            true
        },
        _ => false
    });
}

#[test]
fn one_piece_post() {
    let mut h = DebugHandler::new();
    let mut p = Parser::new(StreamType::Request);

    assert!(match p.parse(&mut h, b"POST ") {
        Ok(Success::Eof(5)) => {
            assert_eq!(h.method, b"POST");
            assert_eq!(p.get_state(), State::StripRequestUrl);
            true
        },
        _ => false
    });
}

#[test]
fn one_piece_put() {
    let mut h = DebugHandler::new();
    let mut p = Parser::new(StreamType::Request);

    assert!(match p.parse(&mut h, b"PUT ") {
        Ok(Success::Eof(4)) => {
            assert_eq!(h.method, b"PUT");
            assert_eq!(p.get_state(), State::StripRequestUrl);
            true
        },
        _ => false
    });
}

#[test]
fn one_piece_trace() {
    let mut h = DebugHandler::new();
    let mut p = Parser::new(StreamType::Request);

    assert!(match p.parse(&mut h, b"TRACE ") {
        Ok(Success::Eof(6)) => {
            assert_eq!(h.method, b"TRACE");
            assert_eq!(p.get_state(), State::StripRequestUrl);
            true
        },
        _ => false
    });
}

#[test]
fn one_piece_unknown() {
    let mut h = DebugHandler::new();
    let mut p = Parser::new(StreamType::Request);

    assert!(match p.parse(&mut h, b"UNKNOWN ") {
        Ok(Success::Eof(8)) => {
            assert_eq!(h.method, b"UNKNOWN");
            assert_eq!(p.get_state(), State::StripRequestUrl);
            true
        },
        _ => false
    });
}

#[test]
fn starting_space() {
    let mut h = DebugHandler::new();
    let mut p = Parser::new(StreamType::Request);

    assert!(match p.parse(&mut h, b"   ") {
        Ok(Success::Eof(3)) => {
            assert_eq!(p.get_state(), State::StripRequestMethod);
            true
        },
        _ => false
    });
}
