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

use http2::{ Flags,
             FrameType,
             Parser,
             ParserState };

use test::http2::*;

#[test]
fn push_promise_with_padding() {
    let mut v = Vec::new();

    // frame payload length and type
    pack_u32!(
        v,
        (28 << 8) | 0x5
    );

    // frame frame flags
    pack_u8!(v, 0x8);

    // frame reserved bit and stream id
    pack_u32!(v, 0x7FFFFFFF);

    // pad length
    pack_u8!(
        v,
        10
    );

    // promised stream id
    pack_u32!(
        v,
        0xFFEE
    );

    // header fragment value
    pack_bytes!(
        v,
        b"Hello, world!"
    );

    // padding
    pack_bytes!(
        v,
        b"XXXXXXXXXX"
    );

    let mut h = DebugHandler::new();
    let mut p = Parser::new();

    p.resume(&mut h, &v);

    assert!(Flags::from_u8(h.frame_flags).is_padded());

    assert_eq!(
        FrameType::from_u8(h.frame_type),
        FrameType::PushPromise
    );

    assert_eq!(
        h.push_promise_stream_id,
        0xFFEE
    );

    assert_eq!(
        h.headers_data,
        b"Hello, world!"
    );

    assert_eq!(
        p.state(),
        ParserState::FrameLength1
    );
}

#[test]
fn push_promise_without_padding() {
    let mut v = Vec::new();

    // frame payload length and type
    pack_u32!(
        v,
        (17 << 8) | 0x5
    );

    // frame frame flags
    pack_u8!(v, 0);

    // frame reserved bit and stream id
    pack_u32!(v, 0x7FFFFFFF);

    // promised stream id
    pack_u32!(
        v,
        0xFFEE
    );

    // header fragment value
    pack_bytes!(
        v,
        b"Hello, world!"
    );

    let mut h = DebugHandler::new();
    let mut p = Parser::new();

    p.resume(&mut h, &v);

    assert!(Flags::from_u8(h.frame_flags).is_empty());

    assert_eq!(
        FrameType::from_u8(h.frame_type),
        FrameType::PushPromise
    );

    assert_eq!(
        h.push_promise_stream_id,
        0xFFEE
    );

    assert_eq!(
        h.headers_data,
        b"Hello, world!"
    );

    assert_eq!(
        p.state(),
        ParserState::FrameLength1
    );
}
