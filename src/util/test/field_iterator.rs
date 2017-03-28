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

use util::*;

#[test]
#[should_panic]
fn name_invalid_byte_error() {
    FieldIterator::new(
        b"compr\ression=bzip",
        b';',
        false
    ).on_error(
        |error| {
            if let FieldError::Name(b'\r') = error {
                panic!();
            }
        }
    ).next();
}

#[test]
#[should_panic]
fn value_ending_quote_missing_error() {
    FieldIterator::new(
        b"compression=\"bzip",
        b';',
        false
    ).on_error(
        |error| {
            if let FieldError::Value(b'p') = error {
                panic!();
            }
        }
    ).next();
}

#[test]
#[should_panic]
fn value_invalid_byte_error() {
    FieldIterator::new(
        b"compression=bz\rip",
        b';',
        false
    ).on_error(
        |error| {
            if let FieldError::Value(b'\r') = error {
                panic!();
            }
        }
    ).next();
}

/*
#[test]
#[should_panic]
fn value_invalid_quote_error() {
    FieldIterator::new(
        b"compression=bzip\"",
        b';',
        false
    ).on_error(
        |error| {
            if let FieldError::Value(_) = error {
                panic!();
            }
        }
    ).next();
}
*/
/*
#[test]
fn normalize() {
    for (n, (name, value)) in FieldIterator::new(
        b"COMPRESSION=bzip; BOUNDARY=longrandomboundarystring",
        b';',
        true
    ).enumerate() {
        if n == 0 {
            assert_eq!(
                name,
                "compression"
            );

            assert_eq!(
                value.unwrap(),
                "bzip"
            );
        } else if n == 1 {
            assert_eq!(
                name,
                "boundary"
            );

            assert_eq!(
                value.unwrap(),
                "longrandomboundarystring"
            );
        }
    }
}

#[test]
fn no_normalize() {
    for (n, (name, value)) in FieldIterator::new(
        b"Compression=Bzip; Boundary=Longrandomboundarystring",
        b';',
        false
    ).enumerate() {
        if n == 0 {
            assert_eq!(
                name,
                "Compression"
            );

            assert_eq!(
                value.unwrap(),
                "Bzip"
            );
        } else if n == 1 {
            assert_eq!(
                name,
                "Boundary"
            );

            assert_eq!(
                value.unwrap(),
                "Longrandomboundarystring"
            );
        }
    }
}

#[test]
fn no_value() {
    for (n, (name, value)) in FieldIterator::new(
        b"compression=bzip; field2; field3",
        b';',
        false
    ).enumerate() {
        if n == 0 {
            assert_eq!(
                name,
                "compression"
            );

            assert_eq!(
                value.unwrap(),
                "bzip"
            );
        } else if n == 1 {
            assert_eq!(
                name,
                "field2"
            );

            assert_eq!(
                value,
                None
            );
        } else if n == 2 {
            assert_eq!(
                name,
                "field3"
            );

            assert_eq!(
                value,
                None
            );
        }
    }
}

#[test]
fn quoted() {
    for (n, (name, value)) in FieldIterator::new(
        b"compression=bzip; content-type=\"application\\\"/\\\"json\"",
        b';',
        false
    ).enumerate() {
        if n == 0 {
            assert_eq!(
                name,
                "compression"
            );

            assert_eq!(
                value.unwrap(),
                "bzip"
            );
        } else if n == 1 {
            assert_eq!(
                name,
                "content-type"
            );

            assert_eq!(
                value.unwrap(),
                "application\"/\"json"
            );
        }
    }
}

#[test]
fn value_error() {
    let mut error = None;

    for (_, _) in FieldIterator::new(
        b"compression=bzip\r",
        b';',
        false
    ).on_error(
        |x| {
            error = Some(x);
        }
    ) {
    }

    match error.unwrap() {
        FieldError::Name(_) => panic!(),
        FieldError::Value(x) => assert_eq!(x, b'\r')
    }
}
*/
