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
    () => (
        http1_setup!()
    );
}

#[test]
fn allowed() {
    // skip `H`, otherwise state will change
    for b in alpha_upper_vec().iter().filter(|&x| *x != b'H') {
        let (mut p, mut h) = setup!();

        assert_eos(
            &mut p,
            &mut h,
            &[*b],
            ParserState::RequestMethod,
            [*b].len()
        );

        assert_eos(
            &mut p,
            &mut h,
            b" ",
            ParserState::RequestUrl1,
            b" ".len()
        );

        assert_eq!(
            h.method,
            &[*b]
        );
    }
}

#[test]
fn callback_exit() {
    struct H;
    impl HttpHandler for H {
        fn on_method(&mut self, _: &[u8]) -> bool {
            false
        }
    }

    let mut h = H;
    let mut p = Parser::new();

    assert_callback(
        &mut p,
        &mut h,
        b"X",
        ParserState::RequestMethod,
        b"X".len()
    );
}

#[test]
fn entire_iter() {
    let (mut p, mut h) = setup!();

    iter_assert_eos(
        &mut p,
        &mut h,
        &[(b'M', ParserState::RequestMethod),
          (b'E', ParserState::RequestMethod),
          (b'T', ParserState::RequestMethod),
          (b'H', ParserState::RequestMethod),
          (b'O', ParserState::RequestMethod),
          (b'D', ParserState::RequestMethod),
          (b' ', ParserState::RequestUrl1)]
    );

    assert_eq!(
        h.method,
        b"METHOD"
    );
}

#[test]
fn not_allowed_error() {
    // anything not upper-cased is invalid
    for b in (0..0x41).chain(0x5B..0xFF) {
        let (mut p, mut h) = setup!();

        assert_error(
            &mut p,
            &mut h,
            &[b],
            ParserError::Method(b)
        );
    }
}

#[test]
fn state_request_url() {
    let (mut p, mut h) = setup!();

    iter_assert_eos(
        &mut p,
        &mut h,
        &[(b'M', ParserState::RequestMethod),
          (b' ', ParserState::RequestUrl1)]
    );
}
