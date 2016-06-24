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

#[test]
fn basic() {
    let mut map = HashMap::new();

    query!(map, b"Field=Value", 11);

    assert_eq!("Value", map.get("Field").unwrap());
}

#[test]
fn complex() {
    let mut map = HashMap::new();

    query!(map, b"Field=Value+%20+%21", 19);

    assert_eq!("Value   !", map.get("Field").unwrap());
}

#[test]
fn ending_ampersand() {
    let mut map = HashMap::new();

    query!(map, b"Field=Value&", 12);

    assert_eq!("Value", map.get("Field").unwrap());
}

#[test]
fn ending_equal() {
    query_error!(b"Field=Value=", b'=', QueryError::Value);
}

#[test]
fn ending_hex() {
    let mut map = HashMap::new();

    query!(map, b"Field=Value%21", 14);

    assert_eq!("Value!", map.get("Field").unwrap());
}

#[test]
fn ending_hex_error1() {
    query_error!(b"Field=Value%", b'%', QueryError::Value);
}

#[test]
fn ending_hex_error2() {
    query_error!(b"Field=Value%2", b'%', QueryError::Value);
}

#[test]
fn ending_hex_error3() {
    query_error!(b"Field=Value%2G", b'%', QueryError::Value);
}

#[test]
fn ending_plus() {
    let mut map = HashMap::new();

    query!(map, b"Field=Value+", 12);

    assert_eq!("Value ", map.get("Field").unwrap());
}

#[test]
fn starting_ampersand() {
    let mut map = HashMap::new();

    query!(map, b"Field=&", 7);

    assert_eq!(0, map.get("Field").unwrap().len());
}

#[test]
fn starting_equal_error() {
    query_error!(b"Field==", b'=', QueryError::Value);
}

#[test]
fn starting_hex() {
    let mut map = HashMap::new();

    query!(map, b"Field=%21Value", 14);

    assert_eq!("!Value", map.get("Field").unwrap());
}

#[test]
fn starting_hex_error1() {
    query_error!(b"Field=%", b'%', QueryError::Value);
}

#[test]
fn starting_hex_error2() {
    query_error!(b"Field=%2", b'%', QueryError::Value);
}

#[test]
fn starting_hex_error3() {
    query_error!(b"Field=%2G", b'%', QueryError::Value);
}

#[test]
fn starting_plus() {
    let mut map = HashMap::new();

    query!(map, b"Field=+", 7);

    assert_eq!(" ", map.get("Field").unwrap());
}
