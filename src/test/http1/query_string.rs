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

use handler::*;
use http1::*;

#[test]
fn basic() {
    let mut h = DebugHandler::new();
    let mut p = Parser::new_request();

    assert!(match p.parse_query_string(&mut h, b"Field=Value") {
        Ok(Success::Finished(length)) => {
            assert_eq!(length, 11);
            assert_eq!(h.url_encoded_field, b"Field");
            assert_eq!(h.url_encoded_value, b"Value");
            true
        },
        _ => false
    });
}

#[test]
fn complex() {
    let mut h = DebugHandler::new();
    let mut p = Parser::new_request();

    assert!(match p.parse_query_string(&mut h, b"Field%20+%21=Value%20+%21") {
        Ok(Success::Finished(length)) => {
            assert_eq!(length, 25);
            assert_eq!(h.url_encoded_field, b"Field  !");
            assert_eq!(h.url_encoded_value, b"Value  !");
            true
        },
        _ => false
    });
}

#[test]
fn field_ending_ampersand() {
    let mut h = DebugHandler::new();
    let mut p = Parser::new_request();

    assert!(match p.parse_query_string(&mut h, b"Field&") {
        Ok(Success::Finished(length)) => {
            assert_eq!(length, 6);
            assert_eq!(h.url_encoded_field, b"Field");
            assert_eq!(h.url_encoded_value, b"");
            true
        },
        _ => false
    });
}

#[test]
fn field_ending_equal() {
    let mut h = DebugHandler::new();
    let mut p = Parser::new_request();

    assert!(match p.parse_query_string(&mut h, b"Field=") {
        Ok(Success::Finished(length)) => {
            assert_eq!(length, 6);
            assert_eq!(h.url_encoded_field, b"Field");
            assert_eq!(h.url_encoded_value, b"");
            true
        },
        _ => false
    });
}

#[test]
fn field_ending_hex() {
    let mut h = DebugHandler::new();
    let mut p = Parser::new_request();

    assert!(match p.parse_query_string(&mut h, b"Field%21") {
        Ok(Success::Finished(length)) => {
            assert_eq!(length, 8);
            assert_eq!(h.url_encoded_field, b"Field!");
            true
        },
        _ => false
    });
}

#[test]
fn field_ending_hex_error1() {
    let mut h = DebugHandler::new();
    let mut p = Parser::new_request();

    if let Err(ParserError::HexSequence(byte)) = p.parse_query_string(&mut h, b"Field%") {
        assert_eq!(byte, b'%');
    } else {
        panic!();
    }
}

#[test]
fn field_ending_hex_error2() {
    let mut h = DebugHandler::new();
    let mut p = Parser::new_request();

    if let Err(ParserError::HexSequence(byte)) = p.parse_query_string(&mut h, b"Field%F") {
        assert_eq!(byte, b'%');
    } else {
        panic!();
    }
}

#[test]
fn field_ending_plus() {
    let mut h = DebugHandler::new();
    let mut p = Parser::new_request();

    assert!(match p.parse_query_string(&mut h, b"Field+") {
        Ok(Success::Finished(length)) => {
            assert_eq!(length, 6);
            assert_eq!(h.url_encoded_field, b"Field ");
            true
        },
        _ => false
    });
}

#[test]
fn value_ending_ampersand() {
    let mut h = DebugHandler::new();
    let mut p = Parser::new_request();

    assert!(match p.parse_query_string(&mut h, b"Field=Value&") {
        Ok(Success::Finished(length)) => {
            assert_eq!(length, 12);
            assert_eq!(h.url_encoded_field, b"Field");
            assert_eq!(h.url_encoded_value, b"Value");
            true
        },
        _ => false
    });
}

#[test]
fn value_ending_hex() {
    let mut h = DebugHandler::new();
    let mut p = Parser::new_request();

    assert!(match p.parse_query_string(&mut h, b"Field=Value%21") {
        Ok(Success::Finished(length)) => {
            assert_eq!(length, 14);
            assert_eq!(h.url_encoded_field, b"Field");
            assert_eq!(h.url_encoded_value, b"Value!");
            true
        },
        _ => false
    });
}

#[test]
fn value_ending_hex_error1() {
    let mut h = DebugHandler::new();
    let mut p = Parser::new_request();

    if let Err(ParserError::HexSequence(byte)) = p.parse_query_string(&mut h, b"Field=Value%") {
        assert_eq!(byte, b'%');
    } else {
        panic!();
    }
}

#[test]
fn value_ending_hex_error2() {
    let mut h = DebugHandler::new();
    let mut p = Parser::new_request();

    if let Err(ParserError::HexSequence(byte)) = p.parse_query_string(&mut h, b"Field=Value%F") {
        assert_eq!(byte, b'%');
    } else {
        panic!();
    }
}

#[test]
fn value_ending_plus() {
    let mut h = DebugHandler::new();
    let mut p = Parser::new_request();

    assert!(match p.parse_query_string(&mut h, b"Field=Value+") {
        Ok(Success::Finished(length)) => {
            assert_eq!(length, 12);
            assert_eq!(h.url_encoded_field, b"Field");
            assert_eq!(h.url_encoded_value, b"Value ");
            true
        },
        _ => false
    });
}

#[test]
fn value_equal_error() {
    let mut h = DebugHandler::new();
    let mut p = Parser::new_request();

    if let Err(ParserError::UrlEncodedValue(byte)) = p.parse_query_string(&mut h, b"Field=Value=") {
        assert_eq!(byte, b'=');
    } else {
        panic!();
    }
}
