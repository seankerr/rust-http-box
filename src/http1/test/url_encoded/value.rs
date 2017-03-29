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

use http1::*;
use http1::test::*;
use test::*;

macro_rules! setup {
    () => ({
        let (mut p, mut h) = http1_setup!();

        p.init_url_encoded();
        p.set_length(1000);

        assert_eos(
            &mut p,
            &mut h,
            b"N=",
            ParserState::UrlEncodedValue,
            b"N=".len()
        );

        (p, h)
    });
}

#[test]
fn allowed() {
    // skip `%`, `+`, `&`, and `;`, otherwise it will change state
    for b in visible_7bit_vec().iter()
                               .filter(|&x| *x != b'%')
                               .filter(|&x| *x != b'+')
                               .filter(|&x| *x != b'&')
                               .filter(|&x| *x != b';') {
        let (mut p, mut h) = setup!();

        p.init_url_encoded();
        p.set_length(1000);

        assert_eos(
            &mut p,
            &mut h,
            b"N=",
            ParserState::UrlEncodedValue,
            b"N=".len()
        );

        assert_eos(
            &mut p,
            &mut h,
            &[*b],
            ParserState::UrlEncodedValue,
            [*b].len()
        );

        assert_eq!(
            h.url_encoded_value,
            &[*b]
        );
    }
}

#[test]
fn entire_iter() {
    let (mut p, mut h) = setup!();

    iter_assert_eos(
        &mut p,
        &mut h,
        &[(b'V', ParserState::UrlEncodedValue),
          (b'a', ParserState::UrlEncodedValue),
          (b'l', ParserState::UrlEncodedValue),
          (b'u', ParserState::UrlEncodedValue),
          (b'e', ParserState::UrlEncodedValue),
          (b'+', ParserState::UrlEncodedValuePlus),
          (b'1', ParserState::UrlEncodedValue),
          (b'%', ParserState::UrlEncodedValueHex1),
          (b'2', ParserState::UrlEncodedValueHex1),
          (b'1', ParserState::UrlEncodedValue),
          (b'V', ParserState::UrlEncodedValue),
          (b'a', ParserState::UrlEncodedValue),
          (b'l', ParserState::UrlEncodedValue),
          (b'u', ParserState::UrlEncodedValue),
          (b'e', ParserState::UrlEncodedValue),
          (b'%', ParserState::UrlEncodedValueHex1),
          (b'2', ParserState::UrlEncodedValueHex2),
          (b'0', ParserState::UrlEncodedValue),
          (b'2', ParserState::UrlEncodedValue),
          (b'%', ParserState::UrlEncodedValueHex1),
          (b'2', ParserState::UrlEncodedValueHex1),
          (b'1', ParserState::UrlEncodedValue)]
    );

    assert_eq!(
        h.url_encoded_value,
        b"Value 1!Value 2!"
    );
}

#[test]
fn finished() {
    let (mut p, mut h) = http1_setup!();

    p.init_url_encoded();
    p.set_length(b"Name+1%21=Value%201%21".len());

    assert_finished(
        &mut p,
        &mut h,
        b"Name+1%21=Value%201%21",
        b"Name+1%21=Value%201%21".len()
    );

    assert_eq!(
        &h.url_encoded_name,
        b"Name 1!"
    );

    assert_eq!(
        &h.url_encoded_value,
        b"Value 1!"
    );
}

#[test]
fn not_allowed_error() {
    for b in non_visible_7bit_vec().iter() {
        let (mut p, mut h) = setup!();

        assert_error(
            &mut p,
            &mut h,
            &[*b],
            ParserError::UrlEncodedValue(*b)
        );
    }
}

#[test]
fn state_url_encoded_value_hex() {
    let (mut p, mut h) = setup!();

    iter_assert_eos(
        &mut p,
        &mut h,
        &[(b'V', ParserState::UrlEncodedValue),
          (b'%', ParserState::UrlEncodedValueHex1),
          (b'2', ParserState::UrlEncodedValueHex2),
          (b'0', ParserState::UrlEncodedValue)]
    );
}

#[test]
fn state_first_url_encoded_name1() {
    let (mut p, mut h) = setup!();

    iter_assert_eos(
        &mut p,
        &mut h,
        &[(b'V', ParserState::UrlEncodedValue),
          (b';', ParserState::FirstUrlEncodedName)]
    );
}

#[test]
fn state_first_url_encoded_name2() {
    let (mut p, mut h) = setup!();

    iter_assert_eos(
        &mut p,
        &mut h,
        &[(b'V', ParserState::UrlEncodedValue),
          (b'&', ParserState::FirstUrlEncodedName)]
    );
}

#[test]
fn state_url_encoded_value_plus() {
    let (mut p, mut h) = setup!();

    iter_assert_eos(
        &mut p,
        &mut h,
        &[(b'V', ParserState::UrlEncodedValue),
          (b'+', ParserState::UrlEncodedValuePlus)]
    );
}
