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

macro_rules! query_error {
    ($stream:expr, $byte:expr) => ({
        assert!(match parse_query($stream, b'&',
                                  |segment| {
                                      match segment {
                                          _ => {}
                                      }
                                  }) {
            Err(QueryError::Field(x)) => {
                assert_eq!(x, $byte);
                true
            },
            _ => false
        });
    });
}

#[test]
fn basic() {
    query!(b"Field", b"Field", b"", true, false, true, 5);
}

#[test]
fn complex() {
    query!(b"Field%20Name+%20+%21", b"Field Name   !", b"", true, false, true, 20);
}

#[test]
fn ending_ampersand() {
    query!(b"Field&", b"Field", b"", true, false, true, 6);
}

#[test]
fn ending_equal() {
    query!(b"Field=", b"Field", b"", true, false, true, 6);
}

#[test]
fn ending_hex() {
    query!(b"Field%21", b"Field!", b"", true, false, true, 8);
}

#[test]
fn ending_hex_error1() {
    query_error!(b"Field%", b'%');
}

#[test]
fn ending_hex_error2() {
    query_error!(b"Field%2", b'%');
}

#[test]
fn ending_hex_error3() {
    query_error!(b"Field%2G", b'%');
}

#[test]
fn ending_plus() {
    query!(b"Field+", b"Field ", b"", true, false, true, 6);
}

#[test]
fn starting_ampersand_error() {
    query_error!(b"&", b'&');
}

#[test]
fn starting_equal_error() {
    query_error!(b"=", b'=');
}

#[test]
fn starting_hex() {
    query!(b"%21Field", b"!Field", b"", true, false, true, 8);
}

#[test]
fn starting_hex_error1() {
    query_error!(b"%", b'%');
}

#[test]
fn starting_hex_error2() {
    query_error!(b"%2", b'%');
}

#[test]
fn starting_hex_error3() {
    query_error!(b"%2G", b'%');
}

#[test]
fn starting_plus() {
    query!(b"+Field", b" Field", b"", true, false, true, 6);
}
