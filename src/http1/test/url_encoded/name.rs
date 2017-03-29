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

        (p, h)
    });
}

#[test]
fn allowed() {
    // skip `=`, `%`, `+`, `&`, and `;`, otherwise it will change state
    for b in visible_7bit_vec().iter()
                               .filter(|&x| *x != b'=')
                               .filter(|&x| *x != b'%')
                               .filter(|&x| *x != b'+')
                               .filter(|&x| *x != b'&')
                               .filter(|&x| *x != b';') {
        let (mut p, mut h) = setup!();

        assert_eos(
            &mut p,
            &mut h,
            &[*b],
            ParserState::UrlEncodedName,
            [*b].len()
        );

        assert_eq!(
            h.url_encoded_name,
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
        &[(b'N', ParserState::UrlEncodedName),
          (b'a', ParserState::UrlEncodedName),
          (b'm', ParserState::UrlEncodedName),
          (b'e', ParserState::UrlEncodedName),
          (b'+', ParserState::UrlEncodedNamePlus),
          (b'1', ParserState::UrlEncodedName),
          (b'%', ParserState::UrlEncodedNameHex1),
          (b'2', ParserState::UrlEncodedNameHex1),
          (b'1', ParserState::UrlEncodedName),
          (b';', ParserState::UrlEncodedName),
          (b'N', ParserState::FirstUrlEncodedName),
          (b'a', ParserState::UrlEncodedName),
          (b'm', ParserState::UrlEncodedName),
          (b'e', ParserState::UrlEncodedName),
          (b'%', ParserState::UrlEncodedNameHex1),
          (b'2', ParserState::UrlEncodedNameHex1),
          (b'0', ParserState::UrlEncodedName),
          (b'2', ParserState::UrlEncodedName),
          (b'%', ParserState::UrlEncodedNameHex1),
          (b'2', ParserState::UrlEncodedNameHex1),
          (b'1', ParserState::UrlEncodedName),
          (b'=', ParserState::UrlEncodedValue)]
    );

    assert_eq!(
        h.url_encoded_name,
        b"Name 1!Name 2!"
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
            ParserError::UrlEncodedName(*b)
        );
    }
}

#[test]
fn state_url_encoded_name_hex() {
    let (mut p, mut h) = setup!();

    iter_assert_eos(
        &mut p,
        &mut h,
        &[(b'N', ParserState::UrlEncodedName),
          (b'%', ParserState::UrlEncodedNameHex1),
          (b'2', ParserState::UrlEncodedNameHex2),
          (b'0', ParserState::UrlEncodedName)]
    );
}

#[test]
fn state_first_url_encoded_name1() {
    let (mut p, mut h) = setup!();

    iter_assert_eos(
        &mut p,
        &mut h,
        &[(b'N', ParserState::UrlEncodedName),
          (b';', ParserState::FirstUrlEncodedName)]
    );
}

#[test]
fn state_first_url_encoded_name2() {
    let (mut p, mut h) = setup!();

    iter_assert_eos(
        &mut p,
        &mut h,
        &[(b'N', ParserState::UrlEncodedName),
          (b'&', ParserState::FirstUrlEncodedName)]
    );
}

#[test]
fn state_url_encoded_name_plus() {
    let (mut p, mut h) = setup!();

    iter_assert_eos(
        &mut p,
        &mut h,
        &[(b'N', ParserState::UrlEncodedName),
          (b'+', ParserState::UrlEncodedNamePlus)]
    );
}

#[test]
fn state_url_encoded_value() {
    let (mut p, mut h) = setup!();

    iter_assert_eos(
        &mut p,
        &mut h,
        &[(b'N', ParserState::UrlEncodedName),
          (b'=', ParserState::UrlEncodedValue)]
    );
}
