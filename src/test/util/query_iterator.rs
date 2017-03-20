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

#[test]
fn hex() {
    let iter = QueryIterator::new(
        b"field%201=value%201&field%202=value%202&field%203"
    );

    for (n, (name, value)) in iter.enumerate() {
        if n == 0 {
            assert_eq!(
                name,
                "field 1"
            );

            assert_eq!(
                value.unwrap(),
                "value 1"
            );
        } else if n == 1 {
            assert_eq!(
                name,
                "field 2"
            );

            assert_eq!(
                value.unwrap(),
                "value 2"
            );
        }  else if n == 2 {
            assert_eq!(
                name,
                "field 3"
            );

            assert_eq!(
                value,
                None
            );
        }
    }
}

#[test]
fn hex_name_error() {
    let mut has_error = false;

    for (_, _) in QueryIterator::new(
        b"field%2Q"
    ).on_error(
        |error| {
            has_error = true;

            match error {
                QueryError::Name(x) => assert_eq!(x, b'Q'),
                QueryError::Value(_) => panic!()
            }
        }
    ) {
    }

    assert!(has_error);
}

#[test]
fn hex_value_error() {
    let mut has_error = false;

    for (_, _) in QueryIterator::new(
        b"field=value%2Q"
    ).on_error(
        |error| {
            has_error = true;

            match error {
                QueryError::Name(_) => panic!(),
                QueryError::Value(x) => assert_eq!(x, b'Q')
            }
        }
    ) {
    }

    assert!(has_error);
}

#[test]
fn no_hex() {
    let iter = QueryIterator::new(
        b"field1=value1&field2=value2&field3"
    );

    for (n, (name, value)) in iter.enumerate() {
        if n == 0 {
            assert_eq!(
                name,
                "field1"
            );

            assert_eq!(
                value.unwrap(),
                "value1"
            );
        } else if n == 1 {
            assert_eq!(
                name,
                "field2"
            );

            assert_eq!(
                value.unwrap(),
                "value2"
            );
        }  else if n == 2 {
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
