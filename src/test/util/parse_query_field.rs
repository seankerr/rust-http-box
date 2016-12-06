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

use util::*;

use std::collections::HashMap;

#[test]
fn basic() {
    let mut map = HashMap::new();

    query!(map, b"Field", 5);

    assert_eq!(0, map.get("Field").unwrap().len());
}

#[test]
fn complex() {
    let mut map = HashMap::new();

    query!(map, b"Field%20Name+%20+%21", 20);

    assert_eq!(0, map.get("Field Name   !").unwrap().len());
}

#[test]
fn ending_ampersand() {
    let mut map = HashMap::new();

    query!(map, b"Field&", 6);

    assert_eq!(0, map.get("Field").unwrap().len());
}

#[test]
fn ending_equal() {
    let mut map = HashMap::new();

    query!(map, b"Field=", 6);

    assert_eq!(0, map.get("Field").unwrap().len());
}

#[test]
fn ending_hex() {
    let mut map = HashMap::new();

    query!(map, b"Field%21", 8);

    assert_eq!(0, map.get("Field!").unwrap().len());
}

#[test]
fn ending_hex_error1() {
    query_error!(b"Field%", b'%', QueryError::Name);
}

#[test]
fn ending_hex_error2() {
    query_error!(b"Field%2", b'2', QueryError::Name);
}

#[test]
fn ending_hex_error3() {
    query_error!(b"Field%2G", b'G', QueryError::Name);
}

#[test]
fn ending_plus() {
    let mut map = HashMap::new();

    query!(map, b"Field+", 6);

    assert_eq!(0, map.get("Field ").unwrap().len());
}

#[test]
fn starting_ampersand_error() {
    query_error!(b"&", b'&', QueryError::Name);
}

#[test]
fn starting_equal_error() {
    query_error!(b"=", b'=', QueryError::Name);
}

#[test]
fn starting_hex() {
    let mut map = HashMap::new();

    query!(map, b"%21Field", 8);

    assert_eq!(0, map.get("!Field").unwrap().len());
}

#[test]
fn starting_hex_error1() {
    query_error!(b"%", b'%', QueryError::Name);
}

#[test]
fn starting_hex_error2() {
    query_error!(b"%2", b'2', QueryError::Name);
}

#[test]
fn starting_hex_error3() {
    query_error!(b"%2G", b'G', QueryError::Name);
}

#[test]
fn starting_plus() {
    let mut map = HashMap::new();

    query!(map, b"+Field", 6);

    assert_eq!(0, map.get(" Field").unwrap().len());
}
