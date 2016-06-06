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

use query::*;

macro_rules! query_error {
    ($stream:expr, $byte:expr) => ({
        assert!(match parse_query($stream, b'&',
                                  |segment| {
                                      match segment {
                                          _ => {}
                                      }
                                  }) {
            Err(QueryError::Value(x)) => {
                assert_eq!(x, $byte);
                true
            },
            _ => false
        });
    });
}

#[test]
fn basic() {
    query!(b"Field=Value", b"Field", b"Value", true, true, true, 11);
}

#[test]
fn complex() {
    query!(b"Field=Value+%20+%21", b"Field", b"Value   !", true, true, true, 19);
}

#[test]
fn ending_ampersand() {
    query!(b"Field=Value&", b"Field", b"Value", true, true, true, 12);
}

#[test]
fn ending_equal() {
    query_error!(b"Field=Value=", b'=');
}

#[test]
fn ending_hex() {
    query!(b"Field=Value%21", b"Field", b"Value!", true, true, true, 14);
}

#[test]
fn ending_hex_error1() {
    query_error!(b"Field=Value%", b'%');
}

#[test]
fn ending_hex_error2() {
    query_error!(b"Field=Value%2", b'%');
}

#[test]
fn ending_hex_error3() {
    query_error!(b"Field=Value%2G", b'%');
}

#[test]
fn ending_plus() {
    query!(b"Field=Value+", b"Field", b"Value ", true, true, true, 12);
}

#[test]
fn starting_ampersand() {
    query!(b"Field=&", b"Field", b"", true, false, true, 7);
}

#[test]
fn starting_equal_error() {
    query_error!(b"Field==", b'=');
}

#[test]
fn starting_hex() {
    query!(b"Field=%21Value", b"Field", b"!Value", true, true, true, 14);
}

#[test]
fn starting_hex_error1() {
    query_error!(b"Field=%", b'%');
}

#[test]
fn starting_hex_error2() {
    query_error!(b"Field=%2", b'%');
}

#[test]
fn starting_hex_error3() {
    query_error!(b"Field=%2G", b'%');
}

#[test]
fn starting_plus() {
    query!(b"Field=+", b"Field", b" ", true, true, true, 7);
}
