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

#[test]
fn decode_without_hex() {
    let mut vec = vec![];

    assert!(match decode(b"justsomedata", &mut vec) {
        Ok(byte_count) => {
            assert_eq!(b"justsomedata", vec.as_slice());
            assert_eq!(12, byte_count);
            true
        },
        _ => false
    });
}

#[test]
fn decode_with_hex() {
    let mut vec = vec![];

    assert!(match decode(b"just%20some%20data", &mut vec) {
        Ok(byte_count) => {
            assert_eq!(b"just some data", vec.as_slice());
            assert_eq!(18, byte_count);
            true
        },
        _ => false
    });
}

#[test]
fn decode_starting_hex() {
    let mut vec = vec![];

    assert!(match decode(b"%20just%20some%20data", &mut vec) {
        Ok(byte_count) => {
            assert_eq!(b" just some data", vec.as_slice());
            assert_eq!(21, byte_count);
            true
        },
        _ => false
    });
}

#[test]
fn decode_ending_hex() {
    let mut vec = vec![];

    assert!(match decode(b"just%20some%20data%20", &mut vec) {
        Ok(byte_count) => {
            assert_eq!(b"just some data ", vec.as_slice());
            assert_eq!(21, byte_count);
            true
        },
        _ => false
    });
}

#[test]
fn decode_sequence() {
    let mut vec = vec![];

    assert!(match decode(b"%20%20just%20some%20data%20%20", &mut vec) {
        Ok(byte_count) => {
            assert_eq!(b"  just some data  ", vec.as_slice());
            assert_eq!(30, byte_count);
            true
        },
        _ => false
    });

    vec.clear();

    assert!(match decode(b"just%20%20%20some%20%20%20data", &mut vec) {
        Ok(byte_count) => {
            assert_eq!(b"just   some   data", vec.as_slice());
            assert_eq!(30, byte_count);
            true
        },
        _ => false
    });
}

#[test]
fn decode_empty() {
    let mut vec = vec![];

    assert!(match decode(b"", &mut vec) {
        Ok(byte_count) => {
            assert_eq!(b"", vec.as_slice());
            assert_eq!(0, byte_count);
            true
        },
        _ => false
    });
}

#[test]
fn decode_invalid_hex_sequence() {
    let mut vec = vec![];

    assert!(match decode(b"%zrjust%20some%20data", &mut vec) {
        Err(DecodingError::Hex(_,_)) => true,
        _                            => false
    });

    vec.clear();

    assert!(match decode(b"just%20so%3qme%20data", &mut vec) {
        Err(DecodingError::Hex(_,_)) => true,
        _                            => false
    });

    vec.clear();

    assert!(match decode(b"just%20some%20data%ag", &mut vec) {
        Err(DecodingError::Hex(_,_)) => true,
        _                            => false
    });
}

#[test]
fn decode_short_hex_sequence() {
    let mut vec = vec![];

    assert!(match decode(b"%", &mut vec) {
        Ok(byte_count) => {
            assert_eq!(0, byte_count);
            true
        },
        _ => false
    });

    vec.clear();

    assert!(match decode(b"ab%", &mut vec) {
        Ok(byte_count) => {
            assert_eq!(b"ab", vec.as_slice());
            assert_eq!(2, byte_count);
            true
        },
        _ => false
    });

    vec.clear();

    assert!(match decode(b"%f", &mut vec) {
        Ok(byte_count) => {
            assert_eq!(b"", vec.as_slice());
            assert_eq!(0, byte_count);
            true
        },
        _ => false
    });

    vec.clear();

    assert!(match decode(b"ab%f", &mut vec) {
        Ok(byte_count) => {
            assert_eq!(b"ab", vec.as_slice());
            assert_eq!(2, byte_count);
            true
        },
        _ => false
    });
}
