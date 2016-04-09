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

use http1::parser::*;

struct H {
    data: Vec<u8>
}

impl HttpHandler for H {
    fn on_method(&mut self, data: &[u8]) -> bool {
        self.data.extend_from_slice(data);
        true
    }
}

#[test]
fn request_method_eof() {
    let mut h = H{data: Vec::new()};
    let mut p = Parser::new(StreamType::Request);

    assert!(match p.parse(&mut h, b"GET") {
        Err(ParserError::Eof) => true,
        _                     => false
    });

    assert_eq!(h.data, b"GET");
    assert_eq!(p.get_state(), State::RequestMethod);
}

#[test]
fn request_method_connect() {
    let mut h = H{data: Vec::new()};
    let mut p = Parser::new(StreamType::Request);

    // slow individual bytes
    assert!(match p.parse(&mut h, b"CO") {
        Err(ParserError::Eof) => true,
        _                     => false
    });

    assert_eq!(h.data, b"CO");
    assert_eq!(p.get_state(), State::RequestMethod);

    assert!(match p.parse(&mut h, b"NNECT ") {
        Err(ParserError::Eof) => true,
        _                     => false
    });

    assert_eq!(h.data, b"CONNECT");
    assert_eq!(p.get_state(), State::RequestUrl);

    // fast chunk
    p.reset();
    h.data = Vec::new();

    assert!(match p.parse(&mut h, b"CONNECT ") {
        Err(ParserError::Eof) => true,
        _                     => false
    });

    assert_eq!(h.data, b"CONNECT");
    assert_eq!(p.get_state(), State::RequestUrl);
}

#[test]
fn request_method_delete() {
    let mut h = H{data: Vec::new()};
    let mut p = Parser::new(StreamType::Request);

    // slow individual bytes
    assert!(match p.parse(&mut h, b"DE") {
        Err(ParserError::Eof) => true,
        _                     => false
    });

    assert_eq!(h.data, b"DE");
    assert_eq!(p.get_state(), State::RequestMethod);

    assert!(match p.parse(&mut h, b"LETE ") {
        Err(ParserError::Eof) => true,
        _                     => false
    });

    assert_eq!(h.data, b"DELETE");
    assert_eq!(p.get_state(), State::RequestUrl);

    // fast chunk
    p.reset();
    h.data = Vec::new();

    assert!(match p.parse(&mut h, b"DELETE ") {
        Err(ParserError::Eof) => true,
        _                     => false
    });

    assert_eq!(h.data, b"DELETE");
    assert_eq!(p.get_state(), State::RequestUrl);
}

#[test]
fn request_method_get() {
    let mut h = H{data: Vec::new()};
    let mut p = Parser::new(StreamType::Request);

    // slow individual bytes
    assert!(match p.parse(&mut h, b"GE") {
        Err(ParserError::Eof) => true,
        _                     => false
    });

    assert_eq!(h.data, b"GE");
    assert_eq!(p.get_state(), State::RequestMethod);

    assert!(match p.parse(&mut h, b"T ") {
        Err(ParserError::Eof) => true,
        _                     => false
    });

    assert_eq!(h.data, b"GET");
    assert_eq!(p.get_state(), State::RequestUrl);

    // fast chunk
    p.reset();
    h.data = Vec::new();

    assert!(match p.parse(&mut h, b"GET ") {
        Err(ParserError::Eof) => true,
        _                     => false
    });

    assert_eq!(h.data, b"GET");
    assert_eq!(p.get_state(), State::RequestUrl);
}

#[test]
fn request_method_head() {
    let mut h = H{data: Vec::new()};
    let mut p = Parser::new(StreamType::Request);

    // slow individual bytes
    assert!(match p.parse(&mut h, b"HE") {
        Err(ParserError::Eof) => true,
        _                     => false
    });

    assert_eq!(h.data, b"HE");
    assert_eq!(p.get_state(), State::RequestMethod);

    assert!(match p.parse(&mut h, b"AD ") {
        Err(ParserError::Eof) => true,
        _                     => false
    });

    assert_eq!(h.data, b"HEAD");
    assert_eq!(p.get_state(), State::RequestUrl);

    // fast chunk
    p.reset();
    h.data = Vec::new();

    assert!(match p.parse(&mut h, b"HEAD ") {
        Err(ParserError::Eof) => true,
        _                     => false
    });

    assert_eq!(h.data, b"HEAD");
    assert_eq!(p.get_state(), State::RequestUrl);
}

#[test]
fn request_method_options() {
    let mut h = H{data: Vec::new()};
    let mut p = Parser::new(StreamType::Request);

    // slow individual bytes
    assert!(match p.parse(&mut h, b"OP") {
        Err(ParserError::Eof) => true,
        _                     => false
    });

    assert_eq!(h.data, b"OP");
    assert_eq!(p.get_state(), State::RequestMethod);

    assert!(match p.parse(&mut h, b"TIONS ") {
        Err(ParserError::Eof) => true,
        _                     => false
    });

    assert_eq!(h.data, b"OPTIONS");
    assert_eq!(p.get_state(), State::RequestUrl);

    // fast chunk
    p.reset();
    h.data = Vec::new();

    assert!(match p.parse(&mut h, b"OPTIONS ") {
        Err(ParserError::Eof) => true,
        _                     => false
    });

    assert_eq!(h.data, b"OPTIONS");
    assert_eq!(p.get_state(), State::RequestUrl);
}

#[test]
fn request_method_post() {
    let mut h = H{data: Vec::new()};
    let mut p = Parser::new(StreamType::Request);

    // slow individual bytes
    assert!(match p.parse(&mut h, b"PO") {
        Err(ParserError::Eof) => true,
        _                     => false
    });

    assert_eq!(h.data, b"PO");
    assert_eq!(p.get_state(), State::RequestMethod);

    assert!(match p.parse(&mut h, b"ST ") {
        Err(ParserError::Eof) => true,
        _                     => false
    });

    assert_eq!(h.data, b"POST");
    assert_eq!(p.get_state(), State::RequestUrl);

    // fast chunk
    p.reset();
    h.data = Vec::new();

    assert!(match p.parse(&mut h, b"POST ") {
        Err(ParserError::Eof) => true,
        _                     => false
    });

    assert_eq!(h.data, b"POST");
    assert_eq!(p.get_state(), State::RequestUrl);
}

#[test]
fn request_method_put() {
    let mut h = H{data: Vec::new()};
    let mut p = Parser::new(StreamType::Request);

    // slow individual bytes
    assert!(match p.parse(&mut h, b"PU") {
        Err(ParserError::Eof) => true,
        _                     => false
    });

    assert_eq!(h.data, b"PU");
    assert_eq!(p.get_state(), State::RequestMethod);

    assert!(match p.parse(&mut h, b"T ") {
        Err(ParserError::Eof) => true,
        _                     => false
    });

    assert_eq!(h.data, b"PUT");
    assert_eq!(p.get_state(), State::RequestUrl);

    // fast chunk
    p.reset();
    h.data = Vec::new();

    assert!(match p.parse(&mut h, b"PUT ") {
        Err(ParserError::Eof) => true,
        _                     => false
    });

    assert_eq!(h.data, b"PUT");
    assert_eq!(p.get_state(), State::RequestUrl);
}

#[test]
fn request_method_trace() {
    let mut h = H{data: Vec::new()};
    let mut p = Parser::new(StreamType::Request);

    // slow individual bytes
    assert!(match p.parse(&mut h, b"TR") {
        Err(ParserError::Eof) => true,
        _                     => false
    });

    assert_eq!(h.data, b"TR");
    assert_eq!(p.get_state(), State::RequestMethod);

    assert!(match p.parse(&mut h, b"ACE ") {
        Err(ParserError::Eof) => true,
        _                     => false
    });

    assert_eq!(h.data, b"TRACE");
    assert_eq!(p.get_state(), State::RequestUrl);

    // fast chunk
    p.reset();
    h.data = Vec::new();

    assert!(match p.parse(&mut h, b"TRACE ") {
        Err(ParserError::Eof) => true,
        _                     => false
    });

    assert_eq!(h.data, b"TRACE");
    assert_eq!(p.get_state(), State::RequestUrl);
}

#[test]
fn request_method_multiple_streams() {
    let mut h = H{data: Vec::new()};
    let mut p = Parser::new(StreamType::Request);

    assert!(match p.parse(&mut h, b"G") {
        Err(ParserError::Eof) => true,
        _                     => false
    });

    assert_eq!(h.data, b"G");
    assert_eq!(p.get_state(), State::RequestMethod);

    assert!(match p.parse(&mut h, b"E") {
        Err(ParserError::Eof) => true,
        _                     => false
    });

    assert_eq!(h.data, b"GE");
    assert_eq!(p.get_state(), State::RequestMethod);

    assert!(match p.parse(&mut h, b"T") {
        Err(ParserError::Eof) => true,
        _                     => false
    });

    assert_eq!(h.data, b"GET");
    assert_eq!(p.get_state(), State::RequestMethod);

    assert!(match p.parse(&mut h, b" ") {
        Err(ParserError::Eof) => true,
        _                     => false
    });

    assert_eq!(h.data, b"GET");
    assert_eq!(p.get_state(), State::RequestUrl);
}

#[test]
fn request_method_invalid_byte() {
    let mut h = H{data: Vec::new()};
    let mut p = Parser::new(StreamType::Request);

    assert!(match p.parse(&mut h, b"G@T") {
        Err(ParserError::Method(_,_)) => true,
        _                             => false
    });

    assert_eq!(p.get_state(), State::Dead);
}
