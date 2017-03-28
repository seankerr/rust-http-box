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

use http2::{ Flags,
             FrameType,
             Parser,
             ParserState };

use http2::test::*;

#[test]
fn all_flags() {
    let mut v = Vec::new();

    // frame payload length and type
    pack_u32!(
        v,
        (
            255 // data length
        ) << 8
        | 0x0 // frame type
    );

    // frame frame flags
    pack_u8!(v, 0xFF);

    // frame reserved bit and stream id
    pack_u32!(v, 0x7FFFFFFF);

    let mut h = DebugHandler::new();
    let mut p = Parser::new();

    p.resume(&mut h, &v);

    assert_eq!(
        255,
        h.frame_payload_length
    );

    assert_eq!(
        FrameType::from_u8(h.frame_type),
        FrameType::Data
    );

    assert!(Flags::from_u8(h.frame_flags).is_end_headers());
    assert!(Flags::from_u8(h.frame_flags).is_end_stream());
    assert!(Flags::from_u8(h.frame_flags).is_padded());
    assert!(Flags::from_u8(h.frame_flags).is_priority());

    assert_eq!(
        h.frame_stream_id,
        0x7FFFFFFF
    );
}

#[test]
fn no_flags() {
    let mut v = Vec::new();

    // frame payload length and type
    pack_u32!(
        v,
        (
            255 // data length
        ) << 8
        | 0x0 // frame type
    );

    // frame frame flags
    pack_u8!(v, 0);

    // frame reserved bit and stream id
    pack_u32!(v, 0x7FFFFFFF);

    let mut h = DebugHandler::new();
    let mut p = Parser::new();

    p.resume(&mut h, &v);

    assert_eq!(
        255,
        h.frame_payload_length
    );

    assert_eq!(
        FrameType::from_u8(h.frame_type),
        FrameType::Data
    );

    assert!(Flags::from_u8(h.frame_flags).is_empty());

    assert_eq!(
        h.frame_stream_id,
        0x7FFFFFFF
    );
}
