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

        assert_eos(
            &mut p,
            &mut h,
            b"HTTP/1.1 200 ",
            ParserState::ResponseStatus1,
            b"HTTP/1.1 200 ".len()
        );

        (p, h)
    });
}

#[test]
fn allowed() {
    for b in non_control_vec().iter() {
        let (mut p, mut h) = setup!();

        assert_eos(
            &mut p,
            &mut h,
            b"X",
            ParserState::ResponseStatus2,
            b"X".len()
        );

        assert_eos(
            &mut p,
            &mut h,
            &[*b],
            ParserState::ResponseStatus2,
            [b].len()
        );
    }
}

#[test]
fn entire_iter() {
    let (mut p, mut h) = setup!();

    iter_assert_eos(
        &mut p,
        &mut h,
        &[(b'N', ParserState::ResponseStatus2),
          (b'O', ParserState::ResponseStatus2),
          (b'T', ParserState::ResponseStatus2),
          (b' ', ParserState::ResponseStatus2),
          (b'F', ParserState::ResponseStatus2),
          (b'O', ParserState::ResponseStatus2),
          (b'U', ParserState::ResponseStatus2),
          (b'N', ParserState::ResponseStatus2),
          (b'D', ParserState::ResponseStatus2),
          (b'\r', ParserState::InitialLf)]
    );

    assert_eq!(
        h.status,
        b"NOT FOUND"
    );
}

#[test]
fn not_allowed_error() {
    // skip `\r`, otherwise state will change
    for b in control_vec().iter().filter(|&x| *x != b'\r') {
        let (mut p, mut h) = setup!();

        assert_eos(
            &mut p,
            &mut h,
            b"X",
            ParserState::ResponseStatus2,
            b"X".len()
        );

        assert_error(
            &mut p,
            &mut h,
            &[*b],
            ParserError::Status(*b)
        );
    }
}

#[test]
fn state_lower_header_name() {
    let (mut p, mut h) = setup!();

    iter_assert_eos(
        &mut p,
        &mut h,
        &[(b'O', ParserState::ResponseStatus2),
          (b'K', ParserState::ResponseStatus2),
          (b'\r', ParserState::InitialLf),
          (b'\n', ParserState::HeaderCr2),
          (b'H', ParserState::LowerHeaderName)]
    );
}
