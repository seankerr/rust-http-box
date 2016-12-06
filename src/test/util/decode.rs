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

use test::*;
use util::*;

#[test]
fn basic() {
    let mut vec = vec![];

    assert!(match decode(b"basic_string", |slice| vec.extend_from_slice(slice)) {
        Ok(12) => {
            assert_eq!(vec, b"basic_string");
            true
        },
        _ => false
    });
}

#[test]
fn byte_check() {
    // invalid bytes
    loop_non_visible(b"", |byte| {
        if let Err(DecodeError::Byte(x)) = decode(&[byte], |_|{}) {
            assert_eq!(x, byte);
        } else {
            panic!();
        }
    });

    // valid bytes
    loop_visible(b"%+", |byte| {
        let mut vec = vec![];

        assert!(match decode(&[byte], |slice| vec.extend_from_slice(slice)) {
            Ok(1) => {
                assert_eq!(vec, &[byte]);
                true
            },
            _ => false
        });
    });
}

#[test]
fn complex() {
    let mut vec = vec![];

    assert!(match decode(b"complex+%21+string", |slice| vec.extend_from_slice(slice)) {
        Ok(18) => {
            assert_eq!(vec, b"complex ! string");
            true
        },
        _ => false
    });
}

#[test]
fn ending_hex() {
    let mut vec = vec![];

    assert!(match decode(b"a%21", |slice| vec.extend_from_slice(slice)) {
        Ok(4) => {
            assert_eq!(vec, b"a!");
            true
        },
        _ => false
    });
}

#[test]
fn ending_hex_error() {
    if let Err(DecodeError::HexSequence(x)) = decode(b"a%", |_|{}) {
        assert_eq!(x, b'%');
    } else {
        panic!();
    }

    if let Err(DecodeError::HexSequence(x)) = decode(b"a%2", |_|{}) {
        assert_eq!(x, b'2');
    } else {
        panic!();
    }

    if let Err(DecodeError::HexSequence(x)) = decode(b"a%2G", |_|{}) {
        assert_eq!(x, b'G');
    } else {
        panic!();
    }
}

#[test]
fn ending_plus() {
    let mut vec = vec![];

    assert!(match decode(b"a+", |slice| vec.extend_from_slice(slice)) {
        Ok(2) => {
            assert_eq!(vec, b"a ");
            true
        },
        _ => false
    });
}

#[test]
fn middle_hex() {
    let mut vec = vec![];

    assert!(match decode(b"a%21a", |slice| vec.extend_from_slice(slice)) {
        Ok(5) => {
            assert_eq!(vec, b"a!a");
            true
        },
        _ => false
    });
}

#[test]
fn middle_hex_error() {
    if let Err(DecodeError::HexSequence(x)) = decode(b"a%2Ga", |_|{}) {
        assert_eq!(x, b'G');
    } else {
        panic!();
    }
}

#[test]
fn middle_plus() {
    let mut vec = vec![];

    assert!(match decode(b"a+a", |slice| vec.extend_from_slice(slice)) {
        Ok(3) => {
            assert_eq!(vec, b"a a");
            true
        },
        _ => false
    });
}

#[test]
fn plus() {
    let mut vec = vec![];

    assert!(match decode(b"+", |slice| vec.extend_from_slice(slice)) {
        Ok(1) => {
            assert_eq!(vec, b" ");
            true
        },
        _ => false
    });
}

#[test]
fn starting_hex() {
    let mut vec = vec![];

    assert!(match decode(b"%21a", |slice| vec.extend_from_slice(slice)) {
        Ok(4) => {
            assert_eq!(vec, b"!a");
            true
        },
        _ => false
    });
}

#[test]
fn starting_hex_error() {
    if let Err(DecodeError::HexSequence(x)) = decode(b"%", |_|{}) {
        assert_eq!(x, b'%');
    } else {
        panic!();
    }

    if let Err(DecodeError::HexSequence(x)) = decode(b"%2", |_|{}) {
        assert_eq!(x, b'2');
    } else {
        panic!();
    }

    if let Err(DecodeError::HexSequence(x)) = decode(b"%2G", |_|{}) {
        assert_eq!(x, b'G');
    } else {
        panic!();
    }
}

#[test]
fn starting_plus() {
    let mut vec = vec![];

    assert!(match decode(b"+a", |slice| vec.extend_from_slice(slice)) {
        Ok(2) => {
            assert_eq!(vec, b" a");
            true
        },
        _ => false
    });
}
