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
use http1::*;
use url::*;
use std::str;

struct H {
    data: Vec<u8>
}

impl HttpHandler for H {
    fn on_method(&mut self, data: &[u8]) -> bool {
        println!("on_method: {:?}", str::from_utf8(data).unwrap());
        self.data.extend_from_slice(data);
        true
    }
}

impl ParamHandler for H {}

#[test]
fn invalid_byte() {
    let mut h = H{data: Vec::new()};
    let mut p = Parser::new(StreamType::Request);

    assert!(match p.parse(&mut h, b"G@T") {
        Err(ParserError::Method(_,x)) => {
            assert_eq!(x, b'@');
            assert_eq!(p.get_state(), State::Dead);
            true
        },
        _ => false
    });
}

#[allow(cyclomatic_complexity)]
#[test]
fn multiple_pieces_connect() {
    let mut h = H{data: Vec::new()};
    let mut p = Parser::new(StreamType::Request);

    assert!(match p.parse(&mut h, b"C") {
        Ok(Success::Eof(1)) => {
            assert_eq!(h.data, b"C");
            assert_eq!(p.get_state(), State::RequestMethod);
            true
        },
        _ => false
    });

    assert!(match p.parse(&mut h, b"O") {
        Ok(Success::Eof(1)) => {
            assert_eq!(h.data, b"CO");
            assert_eq!(p.get_state(), State::RequestMethod);
            true
        },
        _ => false
    });

    assert!(match p.parse(&mut h, b"N") {
        Ok(Success::Eof(1)) => {
            assert_eq!(h.data, b"CON");
            assert_eq!(p.get_state(), State::RequestMethod);
            true
        },
        _ => false
    });

    assert!(match p.parse(&mut h, b"N") {
        Ok(Success::Eof(1)) => {
            assert_eq!(h.data, b"CONN");
            assert_eq!(p.get_state(), State::RequestMethod);
            true
        },
        _ => false
    });

    assert!(match p.parse(&mut h, b"E") {
        Ok(Success::Eof(1)) => {
            assert_eq!(h.data, b"CONNE");
            assert_eq!(p.get_state(), State::RequestMethod);
            true
        },
        _ => false
    });

    assert!(match p.parse(&mut h, b"C") {
        Ok(Success::Eof(1)) => {
            assert_eq!(h.data, b"CONNEC");
            assert_eq!(p.get_state(), State::RequestMethod);
            true
        },
        _ => false
    });

    assert!(match p.parse(&mut h, b"T") {
        Ok(Success::Eof(1)) => {
            assert_eq!(h.data, b"CONNECT");
            assert_eq!(p.get_state(), State::RequestMethod);
            true
        },
        _ => false
    });

    assert!(match p.parse(&mut h, b" ") {
        Ok(Success::Eof(1)) => {
            assert_eq!(h.data, b"CONNECT");
            assert_eq!(p.get_state(), State::RequestUrl);
            true
        },
        _ => false
    });
}

#[allow(cyclomatic_complexity)]
#[test]
fn multiple_pieces_delete() {
    let mut h = H{data: Vec::new()};
    let mut p = Parser::new(StreamType::Request);

    assert!(match p.parse(&mut h, b"D") {
        Ok(Success::Eof(1)) => {
            assert_eq!(h.data, b"D");
            assert_eq!(p.get_state(), State::RequestMethod);
            true
        },
        _ => false
    });

    assert!(match p.parse(&mut h, b"E") {
        Ok(Success::Eof(1)) => {
            assert_eq!(h.data, b"DE");
            assert_eq!(p.get_state(), State::RequestMethod);
            true
        },
        _ => false
    });

    assert!(match p.parse(&mut h, b"L") {
        Ok(Success::Eof(1)) => {
            assert_eq!(h.data, b"DEL");
            assert_eq!(p.get_state(), State::RequestMethod);
            true
        },
        _ => false
    });

    assert!(match p.parse(&mut h, b"E") {
        Ok(Success::Eof(1)) => {
            assert_eq!(h.data, b"DELE");
            assert_eq!(p.get_state(), State::RequestMethod);
            true
        },
        _ => false
    });

    assert!(match p.parse(&mut h, b"T") {
        Ok(Success::Eof(1)) => {
            assert_eq!(h.data, b"DELET");
            assert_eq!(p.get_state(), State::RequestMethod);
            true
        },
        _ => false
    });

    assert!(match p.parse(&mut h, b"E") {
        Ok(Success::Eof(1)) => {
            assert_eq!(h.data, b"DELETE");
            assert_eq!(p.get_state(), State::RequestMethod);
            true
        },
        _ => false
    });

    assert!(match p.parse(&mut h, b" ") {
        Ok(Success::Eof(1)) => {
            assert_eq!(h.data, b"DELETE");
            assert_eq!(p.get_state(), State::RequestUrl);
            true
        },
        _ => false
    });
}

#[allow(cyclomatic_complexity)]
#[test]
fn multiple_pieces_get() {
    let mut h = H{data: Vec::new()};
    let mut p = Parser::new(StreamType::Request);

    assert!(match p.parse(&mut h, b"G") {
        Ok(Success::Eof(1)) => {
            assert_eq!(h.data, b"G");
            assert_eq!(p.get_state(), State::RequestMethod);
            true
        },
        _ => false
    });

    assert!(match p.parse(&mut h, b"E") {
        Ok(Success::Eof(1)) => {
            assert_eq!(h.data, b"GE");
            assert_eq!(p.get_state(), State::RequestMethod);
            true
        },
        _ => false
    });

    assert!(match p.parse(&mut h, b"T") {
        Ok(Success::Eof(1)) => {
            assert_eq!(h.data, b"GET");
            assert_eq!(p.get_state(), State::RequestMethod);
            true
        },
        _ => false
    });

    assert!(match p.parse(&mut h, b" ") {
        Ok(Success::Eof(1)) => {
            assert_eq!(h.data, b"GET");
            assert_eq!(p.get_state(), State::RequestUrl);
            true
        },
        _ => false
    });
}

#[allow(cyclomatic_complexity)]
#[test]
fn multiple_pieces_head() {
    let mut h = H{data: Vec::new()};
    let mut p = Parser::new(StreamType::Request);

    assert!(match p.parse(&mut h, b"H") {
        Ok(Success::Eof(1)) => {
            assert_eq!(h.data, b"H");
            assert_eq!(p.get_state(), State::RequestMethod);
            true
        },
        _ => false
    });

    assert!(match p.parse(&mut h, b"E") {
        Ok(Success::Eof(1)) => {
            assert_eq!(h.data, b"HE");
            assert_eq!(p.get_state(), State::RequestMethod);
            true
        },
        _ => false
    });

    assert!(match p.parse(&mut h, b"A") {
        Ok(Success::Eof(1)) => {
            assert_eq!(h.data, b"HEA");
            assert_eq!(p.get_state(), State::RequestMethod);
            true
        },
        _ => false
    });

    assert!(match p.parse(&mut h, b"D") {
        Ok(Success::Eof(1)) => {
            assert_eq!(h.data, b"HEAD");
            assert_eq!(p.get_state(), State::RequestMethod);
            true
        },
        _ => false
    });

    assert!(match p.parse(&mut h, b" ") {
        Ok(Success::Eof(1)) => {
            assert_eq!(h.data, b"HEAD");
            assert_eq!(p.get_state(), State::RequestUrl);
            true
        },
        _ => false
    });
}

#[allow(cyclomatic_complexity)]
#[test]
fn multiple_pieces_options() {
    let mut h = H{data: Vec::new()};
    let mut p = Parser::new(StreamType::Request);

    assert!(match p.parse(&mut h, b"O") {
        Ok(Success::Eof(1)) => {
            assert_eq!(h.data, b"O");
            assert_eq!(p.get_state(), State::RequestMethod);
            true
        },
        _ => false
    });

    assert!(match p.parse(&mut h, b"P") {
        Ok(Success::Eof(1)) => {
            assert_eq!(h.data, b"OP");
            assert_eq!(p.get_state(), State::RequestMethod);
            true
        },
        _ => false
    });

    assert!(match p.parse(&mut h, b"T") {
        Ok(Success::Eof(1)) => {
            assert_eq!(h.data, b"OPT");
            assert_eq!(p.get_state(), State::RequestMethod);
            true
        },
        _ => false
    });

    assert!(match p.parse(&mut h, b"I") {
        Ok(Success::Eof(1)) => {
            assert_eq!(h.data, b"OPTI");
            assert_eq!(p.get_state(), State::RequestMethod);
            true
        },
        _ => false
    });

    assert!(match p.parse(&mut h, b"O") {
        Ok(Success::Eof(1)) => {
            assert_eq!(h.data, b"OPTIO");
            assert_eq!(p.get_state(), State::RequestMethod);
            true
        },
        _ => false
    });

    assert!(match p.parse(&mut h, b"N") {
        Ok(Success::Eof(1)) => {
            assert_eq!(h.data, b"OPTION");
            assert_eq!(p.get_state(), State::RequestMethod);
            true
        },
        _ => false
    });

    assert!(match p.parse(&mut h, b"S") {
        Ok(Success::Eof(1)) => {
            assert_eq!(h.data, b"OPTIONS");
            assert_eq!(p.get_state(), State::RequestMethod);
            true
        },
        _ => false
    });

    assert!(match p.parse(&mut h, b" ") {
        Ok(Success::Eof(1)) => {
            assert_eq!(h.data, b"OPTIONS");
            assert_eq!(p.get_state(), State::RequestUrl);
            true
        },
        _ => false
    });
}

#[allow(cyclomatic_complexity)]
#[test]
fn multiple_pieces_post() {
    let mut h = H{data: Vec::new()};
    let mut p = Parser::new(StreamType::Request);

    assert!(match p.parse(&mut h, b"P") {
        Ok(Success::Eof(1)) => {
            assert_eq!(h.data, b"P");
            assert_eq!(p.get_state(), State::RequestMethod);
            true
        },
        _ => false
    });

    assert!(match p.parse(&mut h, b"O") {
        Ok(Success::Eof(1)) => {
            assert_eq!(h.data, b"PO");
            assert_eq!(p.get_state(), State::RequestMethod);
            true
        },
        _ => false
    });

    assert!(match p.parse(&mut h, b"S") {
        Ok(Success::Eof(1)) => {
            assert_eq!(h.data, b"POS");
            assert_eq!(p.get_state(), State::RequestMethod);
            true
        },
        _ => false
    });

    assert!(match p.parse(&mut h, b"T") {
        Ok(Success::Eof(1)) => {
            assert_eq!(h.data, b"POST");
            assert_eq!(p.get_state(), State::RequestMethod);
            true
        },
        _ => false
    });

    assert!(match p.parse(&mut h, b" ") {
        Ok(Success::Eof(1)) => {
            assert_eq!(h.data, b"POST");
            assert_eq!(p.get_state(), State::RequestUrl);
            true
        },
        _ => false
    });
}

#[allow(cyclomatic_complexity)]
#[test]
fn multiple_pieces_put() {
    let mut h = H{data: Vec::new()};
    let mut p = Parser::new(StreamType::Request);

    assert!(match p.parse(&mut h, b"P") {
        Ok(Success::Eof(1)) => {
            assert_eq!(h.data, b"P");
            assert_eq!(p.get_state(), State::RequestMethod);
            true
        },
        _ => false
    });

    assert!(match p.parse(&mut h, b"U") {
        Ok(Success::Eof(1)) => {
            assert_eq!(h.data, b"PU");
            assert_eq!(p.get_state(), State::RequestMethod);
            true
        },
        _ => false
    });

    assert!(match p.parse(&mut h, b"T") {
        Ok(Success::Eof(1)) => {
            assert_eq!(h.data, b"PUT");
            assert_eq!(p.get_state(), State::RequestMethod);
            true
        },
        _ => false
    });

    assert!(match p.parse(&mut h, b" ") {
        Ok(Success::Eof(1)) => {
            assert_eq!(h.data, b"PUT");
            assert_eq!(p.get_state(), State::RequestUrl);
            true
        },
        _ => false
    });
}

#[allow(cyclomatic_complexity)]
#[test]
fn multiple_pieces_trace() {
    let mut h = H{data: Vec::new()};
    let mut p = Parser::new(StreamType::Request);

    assert!(match p.parse(&mut h, b"T") {
        Ok(Success::Eof(1)) => {
            assert_eq!(h.data, b"T");
            assert_eq!(p.get_state(), State::RequestMethod);
            true
        },
        _ => false
    });

    assert!(match p.parse(&mut h, b"R") {
        Ok(Success::Eof(1)) => {
            assert_eq!(h.data, b"TR");
            assert_eq!(p.get_state(), State::RequestMethod);
            true
        },
        _ => false
    });

    assert!(match p.parse(&mut h, b"A") {
        Ok(Success::Eof(1)) => {
            assert_eq!(h.data, b"TRA");
            assert_eq!(p.get_state(), State::RequestMethod);
            true
        },
        _ => false
    });

    assert!(match p.parse(&mut h, b"C") {
        Ok(Success::Eof(1)) => {
            assert_eq!(h.data, b"TRAC");
            assert_eq!(p.get_state(), State::RequestMethod);
            true
        },
        _ => false
    });

    assert!(match p.parse(&mut h, b"E") {
        Ok(Success::Eof(1)) => {
            assert_eq!(h.data, b"TRACE");
            assert_eq!(p.get_state(), State::RequestMethod);
            true
        },
        _ => false
    });

    assert!(match p.parse(&mut h, b" ") {
        Ok(Success::Eof(1)) => {
            assert_eq!(h.data, b"TRACE");
            assert_eq!(p.get_state(), State::RequestUrl);
            true
        },
        _ => false
    });
}

#[allow(cyclomatic_complexity)]
#[test]
fn multiple_pieces_unknown() {
    let mut h = H{data: Vec::new()};
    let mut p = Parser::new(StreamType::Request);

    assert!(match p.parse(&mut h, b"U") {
        Ok(Success::Eof(1)) => {
            assert_eq!(h.data, b"U");
            assert_eq!(p.get_state(), State::RequestMethod);
            true
        },
        _ => false
    });

    assert!(match p.parse(&mut h, b"N") {
        Ok(Success::Eof(1)) => {
            assert_eq!(h.data, b"UN");
            assert_eq!(p.get_state(), State::RequestMethod);
            true
        },
        _ => false
    });

    assert!(match p.parse(&mut h, b"K") {
        Ok(Success::Eof(1)) => {
            assert_eq!(h.data, b"UNK");
            assert_eq!(p.get_state(), State::RequestMethod);
            true
        },
        _ => false
    });

    assert!(match p.parse(&mut h, b"N") {
        Ok(Success::Eof(1)) => {
            assert_eq!(h.data, b"UNKN");
            assert_eq!(p.get_state(), State::RequestMethod);
            true
        },
        _ => false
    });

    assert!(match p.parse(&mut h, b"O") {
        Ok(Success::Eof(1)) => {
            assert_eq!(h.data, b"UNKNO");
            assert_eq!(p.get_state(), State::RequestMethod);
            true
        },
        _ => false
    });

    assert!(match p.parse(&mut h, b"W") {
        Ok(Success::Eof(1)) => {
            assert_eq!(h.data, b"UNKNOW");
            assert_eq!(p.get_state(), State::RequestMethod);
            true
        },
        _ => false
    });

    assert!(match p.parse(&mut h, b"N") {
        Ok(Success::Eof(1)) => {
            assert_eq!(h.data, b"UNKNOWN");
            assert_eq!(p.get_state(), State::RequestMethod);
            true
        },
        _ => false
    });

    assert!(match p.parse(&mut h, b" ") {
        Ok(Success::Eof(1)) => {
            assert_eq!(h.data, b"UNKNOWN");
            assert_eq!(p.get_state(), State::RequestUrl);
            true
        },
        _ => false
    });
}

#[test]
fn one_piece_connect() {
    let mut h = H{data: Vec::new()};
    let mut p = Parser::new(StreamType::Request);

    assert!(match p.parse(&mut h, b"CONNECT ") {
        Ok(Success::Eof(8)) => {
            assert_eq!(h.data, b"CONNECT");
            assert_eq!(p.get_state(), State::RequestUrl);
            true
        },
        _ => false
    });
}

#[test]
fn one_piece_delete() {
    let mut h = H{data: Vec::new()};
    let mut p = Parser::new(StreamType::Request);

    assert!(match p.parse(&mut h, b"DELETE ") {
        Ok(Success::Eof(7)) => {
            assert_eq!(h.data, b"DELETE");
            assert_eq!(p.get_state(), State::RequestUrl);
            true
        },
        _ => false
    });
}

#[test]
fn one_piece_get() {
    let mut h = H{data: Vec::new()};
    let mut p = Parser::new(StreamType::Request);

    assert!(match p.parse(&mut h, b"GET ") {
        Ok(Success::Eof(4)) => {
            assert_eq!(h.data, b"GET");
            assert_eq!(p.get_state(), State::RequestUrl);
            true
        },
        _ => false
    });
}

#[test]
fn one_piece_head() {
    let mut h = H{data: Vec::new()};
    let mut p = Parser::new(StreamType::Request);

    assert!(match p.parse(&mut h, b"HEAD ") {
        Ok(Success::Eof(5)) => {
            assert_eq!(h.data, b"HEAD");
            assert_eq!(p.get_state(), State::RequestUrl);
            true
        },
        _ => false
    });
}

#[test]
fn one_piece_options() {
    let mut h = H{data: Vec::new()};
    let mut p = Parser::new(StreamType::Request);

    assert!(match p.parse(&mut h, b"OPTIONS ") {
        Ok(Success::Eof(8)) => {
            assert_eq!(h.data, b"OPTIONS");
            assert_eq!(p.get_state(), State::RequestUrl);
            true
        },
        _ => false
    });
}

#[test]
fn one_piece_post() {
    let mut h = H{data: Vec::new()};
    let mut p = Parser::new(StreamType::Request);

    assert!(match p.parse(&mut h, b"POST ") {
        Ok(Success::Eof(5)) => {
            assert_eq!(h.data, b"POST");
            assert_eq!(p.get_state(), State::RequestUrl);
            true
        },
        _ => false
    });
}

#[test]
fn one_piece_put() {
    let mut h = H{data: Vec::new()};
    let mut p = Parser::new(StreamType::Request);

    assert!(match p.parse(&mut h, b"PUT ") {
        Ok(Success::Eof(4)) => {
            assert_eq!(h.data, b"PUT");
            assert_eq!(p.get_state(), State::RequestUrl);
            true
        },
        _ => false
    });
}

#[test]
fn one_piece_trace() {
    let mut h = H{data: Vec::new()};
    let mut p = Parser::new(StreamType::Request);

    assert!(match p.parse(&mut h, b"TRACE ") {
        Ok(Success::Eof(6)) => {
            assert_eq!(h.data, b"TRACE");
            assert_eq!(p.get_state(), State::RequestUrl);
            true
        },
        _ => false
    });
}

#[test]
fn one_piece_unknown() {
    let mut h = H{data: Vec::new()};
    let mut p = Parser::new(StreamType::Request);

    assert!(match p.parse(&mut h, b"UNKNOWN ") {
        Ok(Success::Eof(8)) => {
            assert_eq!(h.data, b"UNKNOWN");
            assert_eq!(p.get_state(), State::RequestUrl);
            true
        },
        _ => false
    });
}
