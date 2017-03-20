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

use test::*;
use util::*;

#[test]
fn basic() {
    match decode(b"basic_string") {
        Ok(s) => {
            assert_eq!(s, "basic_string");
        },
        _ => panic!()
    };
}

#[test]
fn byte_check() {
    // invalid bytes
    loop_non_visible(b"", |byte| {
        if let Err(DecodeError::Byte(x)) = decode(&[byte]) {
            assert_eq!(x, byte);
        } else {
            panic!();
        }
    });

    // valid bytes
    loop_visible(b"%+", |byte| {
        match decode(&[byte]) {
            Ok(s) => {
                assert_eq!(s, format!("{}", byte as char));
            },
            _ => panic!()
        };
    });
}

#[test]
fn complex() {
    match decode(b"complex+%21+string") {
        Ok(s) => {
            assert_eq!(s, "complex ! string");
        },
        _ => panic!()
    };
}

#[test]
fn ending_hex() {
    match decode(b"a%21") {
        Ok(s) => {
            assert_eq!(s, "a!");
        },
        _ => panic!()
    };
}

#[test]
fn ending_hex_error() {
    if let Err(DecodeError::HexSequence(x)) = decode(b"a%") {
        assert_eq!(x, b'%');
    } else {
        panic!();
    }

    if let Err(DecodeError::HexSequence(x)) = decode(b"a%2") {
        assert_eq!(x, b'2');
    } else {
        panic!();
    }

    if let Err(DecodeError::HexSequence(x)) = decode(b"a%2G") {
        assert_eq!(x, b'G');
    } else {
        panic!();
    }
}

#[test]
fn ending_plus() {
    match decode(b"a+") {
        Ok(s) => {
            assert_eq!(s, "a ");
        },
        _ => panic!()
    };
}

#[test]
fn middle_hex() {
    match decode(b"a%21a") {
        Ok(s) => {
            assert_eq!(s, "a!a");
        },
        _ => panic!()
    };
}

#[test]
fn middle_hex_error() {
    if let Err(DecodeError::HexSequence(x)) = decode(b"a%2Ga") {
        assert_eq!(x, b'G');
    } else {
        panic!();
    }
}

#[test]
fn middle_plus() {
    match decode(b"a+a") {
        Ok(s) => {
            assert_eq!(s, "a a");
        },
        _ => panic!()
    };
}

#[test]
fn plus() {
    match decode(b"+") {
        Ok(s) => {
            assert_eq!(s, " ");
        },
        _ => panic!()
    };
}

#[test]
fn starting_hex() {
    match decode(b"%21a") {
        Ok(s) => {
            assert_eq!(s, "!a");
        },
        _ => panic!()
    };
}

#[test]
fn starting_hex_error() {
    if let Err(DecodeError::HexSequence(x)) = decode(b"%") {
        assert_eq!(x, b'%');
    } else {
        panic!();
    }

    if let Err(DecodeError::HexSequence(x)) = decode(b"%2") {
        assert_eq!(x, b'2');
    } else {
        panic!();
    }

    if let Err(DecodeError::HexSequence(x)) = decode(b"%2G") {
        assert_eq!(x, b'G');
    } else {
        panic!();
    }
}

#[test]
fn starting_plus() {
    match decode(b"+a") {
        Ok(s) => {
            assert_eq!(s, " a");
        },
        _ => panic!()
    }
}
