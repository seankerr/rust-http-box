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

use test::*;
use util::*;

#[test]
fn decode_complex() {
    match decode(b"the%20quick+brown%20fox+jumped%20over+the%20lazy+dog%2E") {
        Ok(s) => assert_eq!(s, "the quick brown fox jumped over the lazy dog."),
        _ => panic!()
    };
}

#[test]
fn ending_hex() {
    match decode(b"X%20") {
        Ok(s) => assert_eq!(s, "X "),
        _ => panic!()
    };
}

#[test]
fn ending_hex_error() {
    if let Err(DecodeError::HexSequence(x)) = decode(b"X%") {
        assert_eq!(x, b'%');
    } else {
        panic!();
    }

    if let Err(DecodeError::HexSequence(x)) = decode(b"X%2") {
        assert_eq!(x, b'2');
    } else {
        panic!();
    }

    if let Err(DecodeError::HexSequence(x)) = decode(b"X%2G") {
        assert_eq!(x, b'G');
    } else {
        panic!();
    }
}

#[test]
fn ending_plus() {
    match decode(b"X+") {
        Ok(s) => assert_eq!(s, "X "),
        _ => panic!()
    };
}

#[test]
fn middle_hex() {
    match decode(b"X%20X") {
        Ok(s) => assert_eq!(s, "X X"),
        _ => panic!()
    };
}

#[test]
fn middle_hex_error() {
    if let Err(DecodeError::HexSequence(x)) = decode(b"X%2GX") {
        assert_eq!(x, b'G');
    } else {
        panic!();
    }
}

#[test]
fn middle_plus() {
    match decode(b"X+X") {
        Ok(s) => assert_eq!(s, "X X"),
        _ => panic!()
    };
}

#[test]
fn no_decoding() {
    match decode(b"string") {
        Ok(s) => assert_eq!(s, "string"),
        _ => panic!()
    };
}

#[test]
fn non_visible_7bit_error() {
    for b in non_visible_7bit_vec().iter() {
        if let Err(DecodeError::Byte(x)) = decode(&[*b]) {
            assert_eq!(x, *b);
        } else {
            panic!();
        }
    };
}

#[test]
fn starting_hex() {
    match decode(b"%21") {
        Ok(s) => {
            assert_eq!(s, "!");
        },
        _ => panic!()
    };
}

#[test]
fn starting_hex_error1() {
    if let Err(DecodeError::HexSequence(x)) = decode(b"%") {
        assert_eq!(x, b'%');
    } else {
        panic!();
    }
}

#[test]
fn starting_hex_error2() {
    if let Err(DecodeError::HexSequence(x)) = decode(b"%2") {
        assert_eq!(x, b'2');
    } else {
        panic!();
    }
}

#[test]
fn starting_hex_error3() {
    if let Err(DecodeError::HexSequence(x)) = decode(b"%2G") {
        assert_eq!(x, b'G');
    } else {
        panic!();
    }
}

#[test]
fn starting_plus() {
    match decode(b"+") {
        Ok(s) => assert_eq!(s, " "),
        _ => panic!()
    }
}

#[test]
fn visible_7bit() {
    for b in visible_7bit_vec().iter()
                               .filter(|&x| *x == b'%')
                               .filter(|&x| *x == b'+') {
        match decode(&[*b]) {
            Ok(s) => assert_eq!(s, format!("{}", *b as char)),
            _ => panic!()
        };
    };
}
