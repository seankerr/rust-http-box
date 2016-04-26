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

struct H {
    data: Vec<u8>
}

impl HttpHandler for H {
    fn on_method(&mut self, data: &[u8]) -> bool {
        self.data.extend_from_slice(data);
        true
    }
}

impl ParamHandler for H {}

#[test]
fn request_method_eof() {
    let mut h = H{data: Vec::new()};
    let mut p = Parser::new(StreamType::Request);

    assert!(match p.parse(&mut h, b"GET") {
        Ok(Success::Eof(_)) => true,
        _ => false
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
        Ok(Success::Eof(_)) => true,
        _ => false
    });

    assert_eq!(h.data, b"CO");
    assert_eq!(p.get_state(), State::RequestMethod);

    assert!(match p.parse(&mut h, b"NNECT ") {
        Ok(Success::Eof(_)) => true,
        _ => false
    });

    assert_eq!(h.data, b"CONNECT");
    assert_eq!(p.get_state(), State::RequestUrl);

    // fast chunk
    p.reset();
    h.data = Vec::new();

    assert!(match p.parse(&mut h, b"CONNECT ") {
        Ok(Success::Eof(_)) => true,
        _ => false
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
        Ok(Success::Eof(_)) => true,
        _ => false
    });

    assert_eq!(h.data, b"DE");
    assert_eq!(p.get_state(), State::RequestMethod);

    assert!(match p.parse(&mut h, b"LETE ") {
        Ok(Success::Eof(_)) => true,
        _ => false
    });

    assert_eq!(h.data, b"DELETE");
    assert_eq!(p.get_state(), State::RequestUrl);

    // fast chunk
    p.reset();
    h.data = Vec::new();

    assert!(match p.parse(&mut h, b"DELETE ") {
        Ok(Success::Eof(_)) => true,
        _ => false
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
        Ok(Success::Eof(_)) => true,
        _ => false
    });

    assert_eq!(h.data, b"GE");
    assert_eq!(p.get_state(), State::RequestMethod);

    assert!(match p.parse(&mut h, b"T ") {
        Ok(Success::Eof(_)) => true,
        _ => false
    });

    assert_eq!(h.data, b"GET");
    assert_eq!(p.get_state(), State::RequestUrl);

    // fast chunk
    p.reset();
    h.data = Vec::new();

    assert!(match p.parse(&mut h, b"GET ") {
        Ok(Success::Eof(_)) => true,
        _ => false
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
        Ok(Success::Eof(_)) => true,
        _ => false
    });

    assert_eq!(h.data, b"HE");
    assert_eq!(p.get_state(), State::RequestMethod);

    assert!(match p.parse(&mut h, b"AD ") {
        Ok(Success::Eof(_)) => true,
        _ => false
    });

    assert_eq!(h.data, b"HEAD");
    assert_eq!(p.get_state(), State::RequestUrl);

    // fast chunk
    p.reset();
    h.data = Vec::new();

    assert!(match p.parse(&mut h, b"HEAD ") {
        Ok(Success::Eof(_)) => true,
        _ => false
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
        Ok(Success::Eof(_)) => true,
        _ => false
    });

    assert_eq!(h.data, b"OP");
    assert_eq!(p.get_state(), State::RequestMethod);

    assert!(match p.parse(&mut h, b"TIONS ") {
        Ok(Success::Eof(_)) => true,
        _ => false
    });

    assert_eq!(h.data, b"OPTIONS");
    assert_eq!(p.get_state(), State::RequestUrl);

    // fast chunk
    p.reset();
    h.data = Vec::new();

    assert!(match p.parse(&mut h, b"OPTIONS ") {
        Ok(Success::Eof(_)) => true,
        _ => false
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
        Ok(Success::Eof(_)) => true,
        _ => false
    });

    assert_eq!(h.data, b"PO");
    assert_eq!(p.get_state(), State::RequestMethod);

    assert!(match p.parse(&mut h, b"ST ") {
        Ok(Success::Eof(_)) => true,
        _ => false
    });

    assert_eq!(h.data, b"POST");
    assert_eq!(p.get_state(), State::RequestUrl);

    // fast chunk
    p.reset();
    h.data = Vec::new();

    assert!(match p.parse(&mut h, b"POST ") {
        Ok(Success::Eof(_)) => true,
        _ => false
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
        Ok(Success::Eof(_)) => true,
        _ => false
    });

    assert_eq!(h.data, b"PU");
    assert_eq!(p.get_state(), State::RequestMethod);

    assert!(match p.parse(&mut h, b"T ") {
        Ok(Success::Eof(_)) => true,
        _ => false
    });

    assert_eq!(h.data, b"PUT");
    assert_eq!(p.get_state(), State::RequestUrl);

    // fast chunk
    p.reset();
    h.data = Vec::new();

    assert!(match p.parse(&mut h, b"PUT ") {
        Ok(Success::Eof(_)) => true,
        _ => false
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
        Ok(Success::Eof(_)) => true,
        _ => false
    });

    assert_eq!(h.data, b"TR");
    assert_eq!(p.get_state(), State::RequestMethod);

    assert!(match p.parse(&mut h, b"ACE ") {
        Ok(Success::Eof(_)) => true,
        _ => false
    });

    assert_eq!(h.data, b"TRACE");
    assert_eq!(p.get_state(), State::RequestUrl);

    // fast chunk
    p.reset();
    h.data = Vec::new();

    assert!(match p.parse(&mut h, b"TRACE ") {
        Ok(Success::Eof(_)) => true,
        _ => false
    });

    assert_eq!(h.data, b"TRACE");
    assert_eq!(p.get_state(), State::RequestUrl);
}

#[test]
fn request_method_multiple_streams() {
    let mut h = H{data: Vec::new()};
    let mut p = Parser::new(StreamType::Request);

    assert!(match p.parse(&mut h, b"G") {
        Ok(Success::Eof(_)) => true,
        _ => false
    });

    assert_eq!(h.data, b"G");
    assert_eq!(p.get_state(), State::RequestMethod);

    assert!(match p.parse(&mut h, b"E") {
        Ok(Success::Eof(_)) => true,
        _ => false
    });

    assert_eq!(h.data, b"GE");
    assert_eq!(p.get_state(), State::RequestMethod);

    assert!(match p.parse(&mut h, b"T") {
        Ok(Success::Eof(_)) => true,
        _ => false
    });

    assert_eq!(h.data, b"GET");
    assert_eq!(p.get_state(), State::RequestMethod);

    assert!(match p.parse(&mut h, b" ") {
        Ok(Success::Eof(_)) => true,
        _ => false
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
        _ => false
    });

    assert_eq!(p.get_state(), State::Dead);
}
