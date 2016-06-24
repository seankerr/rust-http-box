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

use util::*;

use std::collections::HashMap;
use std::str;

#[test]
fn missing_escape_byte_error() {
    field_error!(b"name=\"value\\", b'\\', FieldError::Value);
}

#[test]
fn missing_quote_error() {
    field_error!(b"name=\"value", b'e', FieldError::Value);
}

#[test]
fn missing_semicolon_error() {
    field_error!(b"name=\"value\" abc", b'a', FieldError::Value);
}

#[test]
fn no_value_no_semi() {
    let mut map = HashMap::new();

    field!(map, b"name-no-value", 13);

    assert_eq!(0, map.get("name-no-value").unwrap().len());
}

#[test]
fn no_value_with_semi() {
    let mut map = HashMap::new();

    field!(map, b"name-no-value;", 14);

    assert_eq!(0, map.get("name-no-value").unwrap().len());
}

#[test]
fn quoted_escaped() {
    let mut map = HashMap::new();

    field!(map, b"name=\"value \\\"2\\\" here\"", 23);

    assert_eq!("value \"2\" here", map.get("name").unwrap());
}

#[test]
fn quoted_no_semi() {
    let mut map = HashMap::new();

    field!(map, b"name=\"value\"", 12);

    assert_eq!("value", map.get("name").unwrap());
}

#[test]
fn quoted_with_semi() {
    let mut map = HashMap::new();

    field!(map, b"name=\"value\";", 13);

    assert_eq!("value", map.get("name").unwrap());
}

#[test]
fn multiple_no_semi() {
    let mut map = HashMap::new();

    field!(map, b"name-no-value; name1=value1; name2=\"value2\"", 43);

    assert_eq!("", map.get("name-no-value").unwrap());
    assert_eq!("value1", map.get("name1").unwrap());
    assert_eq!("value2", map.get("name2").unwrap());
}

#[test]
fn multiple_with_semi() {
    let mut map = HashMap::new();

    field!(map, b"name-no-value; name1=value1; name2=\"value2\";", 44);

    assert_eq!("", map.get("name-no-value").unwrap());
    assert_eq!("value1", map.get("name1").unwrap());
    assert_eq!("value2", map.get("name2").unwrap());
}

#[test]
fn unquoted_no_semi() {
    let mut map = HashMap::new();

    field!(map, b"name=value", 10);

    assert_eq!("value", map.get("name").unwrap());
}

#[test]
fn unquoted_with_semi() {
    let mut map = HashMap::new();

    field!(map, b"name=value;", 11);

    assert_eq!("value", map.get("name").unwrap());
}
