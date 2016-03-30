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

use url::*;
use std::str;

struct H {
    field: Vec<u8>,
    value: Vec<u8>
}

impl ParamHandler for H {
    fn on_param_field(&mut self, data: &[u8]) -> bool {
        println!("on_param_field: {:?}", str::from_utf8(data).unwrap());
        self.field.extend_from_slice(data);
        true
    }

    fn on_param_value(&mut self, data: &[u8]) -> bool {
        println!("on_param_value: {:?}", str::from_utf8(data).unwrap());
        self.value.extend_from_slice(data);
        true
    }
}

#[test]
fn parse_query_string_single() {
    let mut h = H{field: Vec::new(), value: Vec::new()};

    assert!(match parse_query_string(&mut h, b"param1=value1") {
        Ok(13) => true,
        _      => false
    });

    assert_eq!(h.field, b"param1");
    assert_eq!(h.value, b"value1");
}

#[test]
fn parse_query_string_plus_sign() {
    let mut h = H{field: Vec::new(), value: Vec::new()};

    assert!(match parse_query_string(&mut h, b"param+1=value+1") {
        Ok(15) => true,
        _      => false
    });

    assert_eq!(h.field, b"param 1");
    assert_eq!(h.value, b"value 1");
}

#[test]
fn parse_query_string_single_hex() {
    let mut h = H{field: Vec::new(), value: Vec::new()};

    assert!(match parse_query_string(&mut h, b"param%201=value%201") {
        Ok(19) => true,
        _      => false
    });

    assert_eq!(h.field, b"param 1");
    assert_eq!(h.value, b"value 1");
}

#[test]
fn parse_query_string_multiple() {
    let mut h = H{field: Vec::new(), value: Vec::new()};

    assert!(match parse_query_string(&mut h, b"param1=value1&param2=value2&param3=value3") {
        Ok(41) => true,
        _      => false
    });

    assert_eq!(h.field, b"param1param2param3");
    assert_eq!(h.value, b"value1value2value3");
}

#[test]
fn parse_query_string_multiple_hex() {
    let mut h = H{field: Vec::new(), value: Vec::new()};

    assert!(match parse_query_string(&mut h, b"param%201=value%201&param%202=value%202&param%203=value%203") {
        Ok(59) => true,
        _      => false
    });

    assert_eq!(h.field, b"param 1param 2param 3");
    assert_eq!(h.value, b"value 1value 2value 3");
}

#[test]
fn parse_query_string_starting_hex() {
    let mut h = H{field: Vec::new(), value: Vec::new()};

    assert!(match parse_query_string(&mut h, b"%20param1=%20value1") {
        Ok(19) => true,
        _      => false
    });

    assert_eq!(h.field, b" param1");
    assert_eq!(h.value, b" value1");
}

#[test]
fn parse_query_string_starting_question() {
    let mut h = H{field: Vec::new(), value: Vec::new()};

    assert!(match parse_query_string(&mut h, b"?param1=value1") {
        Ok(14) => true,
        _      => false
    });

    assert_eq!(h.field, b"param1");
    assert_eq!(h.value, b"value1");
}

#[test]
fn parse_query_string_starting_ampersand() {
    let mut h = H{field: Vec::new(), value: Vec::new()};

    assert!(match parse_query_string(&mut h, b"&param1=value1") {
        Ok(14) => true,
        _      => false
    });

    assert_eq!(h.field, b"param1");
    assert_eq!(h.value, b"value1");
}

#[test]
fn parse_query_string_ending_ampersand() {
    let mut h = H{field: Vec::new(), value: Vec::new()};

    assert!(match parse_query_string(&mut h, b"?param1=value1&") {
        Ok(15) => true,
        _      => false
    });

    assert_eq!(h.field, b"param1");
    assert_eq!(h.value, b"value1");

    h = H{field: Vec::new(), value: Vec::new()};

    assert!(match parse_query_string(&mut h, b"?param1=value1&param2=&") {
        Ok(23) => true,
        _      => false
    });

    assert_eq!(h.field, b"param1param2");
    assert_eq!(h.value, b"value1");
}

#[test]
fn parse_query_string_ending_equal() {
    let mut h = H{field: Vec::new(), value: Vec::new()};

    assert!(match parse_query_string(&mut h, b"?param1=value1&param2=") {
        Ok(22) => true,
        _      => false
    });

    assert_eq!(h.field, b"param1param2");
    assert_eq!(h.value, b"value1");
}

#[test]
fn parse_query_string_ending_hex() {
    let mut h = H{field: Vec::new(), value: Vec::new()};

    assert!(match parse_query_string(&mut h, b"param1%20=value1%20") {
        Ok(19) => true,
        _      => false
    });

    assert_eq!(h.field, b"param1 ");
    assert_eq!(h.value, b"value1 ");
}

#[test]
fn parse_query_string_hex_control_character() {
    let mut h = H{field: Vec::new(), value: Vec::new()};

    assert!(match parse_query_string(&mut h, b"param1%0A=value1%0A") {
        Ok(19) => true,
        _      => false
    });

    assert_eq!(h.field, b"param1\n");
    assert_eq!(h.value, b"value1\n");
}

#[test]
fn parse_query_string_invalid_param_field() {
    let mut h = H{field: Vec::new(), value: Vec::new()};

    assert!(match parse_query_string(&mut h, b"param1\r=value1") {
        Err(ParamError::Field(_,_)) => true,
        _                           => false
    });
}

#[test]
fn parse_query_string_invalid_param_value() {
    let mut h = H{field: Vec::new(), value: Vec::new()};

    assert!(match parse_query_string(&mut h, b"param1=value1\r") {
        Err(ParamError::Value(_,_)) => true,
        _                           => false
    });
}
