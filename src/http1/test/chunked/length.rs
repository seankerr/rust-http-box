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

macro_rules! setup {
    () => ({
        let (mut p, h) = http1_setup!();

        p.init_chunked();

        (p, h)
    });
}

#[test]
fn allowed1() {
    let (mut p, mut h) = setup!();

    assert_eos(
        &mut p,
        &mut h,
        b"F\r",
        ParserState::ChunkLengthLf,
        b"F\r".len()
    );

    assert_eq!(
        h.chunk_length,
        0xF
    )
}

#[test]
fn allowed2() {
    let (mut p, mut h) = setup!();

    assert_eos(
        &mut p,
        &mut h,
        b"FF\r",
        ParserState::ChunkLengthLf,
        b"FF\r".len()
    );

    assert_eq!(
        h.chunk_length,
        0xFF
    )
}

#[test]
fn allowed3() {
    let (mut p, mut h) = setup!();

    assert_eos(
        &mut p,
        &mut h,
        b"FFF\r",
        ParserState::ChunkLengthLf,
        b"FFF\r".len()
    );

    assert_eq!(
        h.chunk_length,
        0xFFF
    )
}

#[test]
fn allowed4() {
    let (mut p, mut h) = setup!();

    assert_eos(
        &mut p,
        &mut h,
        b"FFFF\r",
        ParserState::ChunkLengthLf,
        b"FFFF\r".len()
    );

    assert_eq!(
        h.chunk_length,
        0xFFFF
    )
}

#[test]
fn allowed5() {
    let (mut p, mut h) = setup!();

    assert_eos(
        &mut p,
        &mut h,
        b"FFFFF\r",
        ParserState::ChunkLengthLf,
        b"FFFFF\r".len()
    );

    assert_eq!(
        h.chunk_length,
        0xFFFFF
    )
}

#[test]
fn allowed6() {
    let (mut p, mut h) = setup!();

    assert_eos(
        &mut p,
        &mut h,
        b"FFFFFF\r",
        ParserState::ChunkLengthLf,
        b"FFFFFF\r".len()
    );

    assert_eq!(
        h.chunk_length,
        0xFFFFFF
    )
}

#[test]
fn allowed7() {
    let (mut p, mut h) = setup!();

    assert_eos(
        &mut p,
        &mut h,
        b"FFFFFFF\r",
        ParserState::ChunkLengthLf,
        b"FFFFFFF\r".len()
    );

    assert_eq!(
        h.chunk_length,
        0xFFFFFFF
    )
}

#[test]
fn allowed8() {
    let (mut p, mut h) = setup!();

    assert_eos(
        &mut p,
        &mut h,
        b"FFFFFFFF\r",
        ParserState::ChunkLengthLf,
        b"FFFFFFFF\r".len()
    );

    assert_eq!(
        h.chunk_length,
        0xFFFFFFFF
    )
}

#[test]
fn callback() {
    struct H;
    impl HttpHandler for H {
        fn on_chunk_length(&mut self, _: usize) -> bool {
            false
        }
    }

    let mut h = H;
    let mut p = Parser::new();

    p.init_chunked();

    assert_callback(
        &mut p,
        &mut h,
        b"F\r",
        ParserState::ChunkLengthLf,
        b"F\r".len()
    );
}

#[test]
fn entire_iter() {
    let (mut p, mut h) = setup!();

    iter_assert_eos(
        &mut p,
        &mut h,
        &[(b'F', ParserState::ChunkLength2),
          (b'F', ParserState::ChunkLength3),
          (b'F', ParserState::ChunkLength4),
          (b'F', ParserState::ChunkLength5),
          (b'F', ParserState::ChunkLength6),
          (b'F', ParserState::ChunkLength7),
          (b'F', ParserState::ChunkLength8),
          (b'F', ParserState::ChunkLengthCr),
          (b'\r', ParserState::ChunkLengthLf)]
    );

    assert_eq!(
        h.chunk_length,
        0xFFFFFFFF
    );
}

#[test]
fn not_allowed_hex_error1() {
    for b in (0..255).filter(|&x| !is_hex!(x)) {
        let (mut p, mut h) = setup!();

        assert_error(
            &mut p,
            &mut h,
            &[b],
            ParserError::ChunkLength(b)
        );
    }
}

#[test]
fn not_allowed_hex_error2() {
    // skip `\r` and `;`, otherwise state will change
    for b in (0..255).filter(|&x| !is_hex!(x))
                     .filter(|&x| x != b'\r')
                     .filter(|&x| x != b';') {
        let (mut p, mut h) = setup!();

        assert_eos(
            &mut p,
            &mut h,
            b"F",
            ParserState::ChunkLength2,
            b"F".len()
        );

        assert_error(
            &mut p,
            &mut h,
            &[b],
            ParserError::ChunkLength(b)
        );
    }
}

#[test]
fn not_allowed_hex_error3() {
    // skip `\r` and `;`, otherwise state will change
    for b in (0..255).filter(|&x| !is_hex!(x))
                     .filter(|&x| x != b'\r')
                     .filter(|&x| x != b';') {
        let (mut p, mut h) = setup!();

        assert_eos(
            &mut p,
            &mut h,
            b"FF",
            ParserState::ChunkLength3,
            b"FF".len()
        );

        assert_error(
            &mut p,
            &mut h,
            &[b],
            ParserError::ChunkLength(b)
        );
    }
}

#[test]
fn not_allowed_hex_error4() {
    // skip `\r` and `;`, otherwise state will change
    for b in (0..255).filter(|&x| !is_hex!(x))
                     .filter(|&x| x != b'\r')
                     .filter(|&x| x != b';') {
        let (mut p, mut h) = setup!();

        assert_eos(
            &mut p,
            &mut h,
            b"FFF",
            ParserState::ChunkLength4,
            b"FFF".len()
        );

        assert_error(
            &mut p,
            &mut h,
            &[b],
            ParserError::ChunkLength(b)
        );
    }
}

#[test]
fn not_allowed_hex_error5() {
    // skip `\r` and `;`, otherwise state will change
    for b in (0..255).filter(|&x| !is_hex!(x))
                     .filter(|&x| x != b'\r')
                     .filter(|&x| x != b';') {
        let (mut p, mut h) = setup!();

        assert_eos(
            &mut p,
            &mut h,
            b"FFFF",
            ParserState::ChunkLength5,
            b"FFFF".len()
        );

        assert_error(
            &mut p,
            &mut h,
            &[b],
            ParserError::ChunkLength(b)
        );
    }
}

#[test]
fn not_allowed_hex_error6() {
    // skip `\r` and `;`, otherwise state will change
    for b in (0..255).filter(|&x| !is_hex!(x))
                     .filter(|&x| x != b'\r')
                     .filter(|&x| x != b';') {
        let (mut p, mut h) = setup!();

        assert_eos(
            &mut p,
            &mut h,
            b"FFFFF",
            ParserState::ChunkLength6,
            b"FFFFF".len()
        );

        assert_error(
            &mut p,
            &mut h,
            &[b],
            ParserError::ChunkLength(b)
        );
    }
}

#[test]
fn not_allowed_hex_error7() {
    // skip `\r` and `;`, otherwise state will change
    for b in (0..255).filter(|&x| !is_hex!(x))
                     .filter(|&x| x != b'\r')
                     .filter(|&x| x != b';') {
        let (mut p, mut h) = setup!();

        assert_eos(
            &mut p,
            &mut h,
            b"FFFFFF",
            ParserState::ChunkLength7,
            b"FFFFFF".len()
        );

        assert_error(
            &mut p,
            &mut h,
            &[b],
            ParserError::ChunkLength(b)
        );
    }
}

#[test]
fn not_allowed_hex_error8() {
    // skip `\r` and `;`, otherwise state will change
    for b in (0..255).filter(|&x| !is_hex!(x))
                     .filter(|&x| x != b'\r')
                     .filter(|&x| x != b';') {
        let (mut p, mut h) = setup!();

        assert_eos(
            &mut p,
            &mut h,
            b"FFFFFFF",
            ParserState::ChunkLength8,
            b"FFFFFFF".len()
        );

        assert_error(
            &mut p,
            &mut h,
            &[b],
            ParserError::ChunkLength(b)
        );
    }
}

#[test]
fn overflow_error() {
    let (mut p, mut h) = setup!();

    assert_error(
        &mut p,
        &mut h,
        b"FFFFFFFFF",
        ParserError::ChunkLength(b'F')
    );
}

#[test]
fn state_chunk_data() {
    let (mut p, mut h) = setup!();

    iter_assert_eos(
        &mut p,
        &mut h,
        &[(b'F', ParserState::ChunkLength2),
          (b'\r', ParserState::ChunkLengthLf),
          (b'\n', ParserState::ChunkData)]
    );
}

#[test]
fn state_extension_name() {
    let (mut p, mut h) = setup!();

    iter_assert_eos(
        &mut p,
        &mut h,
        &[(b'F', ParserState::ChunkLength2),
          (b';', ParserState::StripChunkExtensionName)]
    );
}
