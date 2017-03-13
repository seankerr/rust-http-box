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
fn is_exclusive() {
    let mut v = Vec::new();

    // frame payload length and type
    pack_u32!(
        v,
        (40 << 8) | 0x2
    );

    // frame frame flags
    pack_u8!(v, 0);

    // frame reserved bit and stream id
    pack_u32!(v, 0x7FFFFFFF);

    // priority exclusive bit stream dependency id
    pack_u32!(
        v,
        (1 << 31) | 42
    );

    // priority weight
    pack_u8!(
        v,
        99
    );

    let mut p = Parser::new(DebugHandler::new());

    p.resume(&v);

    {
        let h = p.handler();

        assert!(Flags::from_u8(h.frame_flags).is_empty());

        assert_eq!(
            FrameType::from_u8(h.frame_type),
            FrameType::Priority
        );

        assert!(h.priority_exclusive);

        assert_eq!(
            h.priority_stream_id,
            42
        );

        assert_eq!(
            h.priority_weight,
            99
        );
    }
    
    assert_eq!(
        p.state(),
        ParserState::FrameLength1
    );
}

#[test]
fn is_not_exclusive() {
    let mut v = Vec::new();

    // frame payload length and type
    pack_u32!(
        v,
        (40 << 8) | 0x2
    );

    // frame frame flags
    pack_u8!(v, 0);

    // frame reserved bit and stream id
    pack_u32!(v, 0x7FFFFFFF);

    // priority exclusive bit stream dependency id
    pack_u32!(
        v,
        (0 << 31) | 42
    );

    // priority weight
    pack_u8!(
        v,
        99
    );

    let mut p = Parser::new(DebugHandler::new());

    p.resume(&v);

    {
        let h = p.handler();

        assert!(Flags::from_u8(h.frame_flags).is_empty());

        assert_eq!(
            FrameType::from_u8(h.frame_type),
            FrameType::Priority
        );

        assert!(!h.priority_exclusive);

        assert_eq!(
            h.priority_stream_id,
            42
        );

        assert_eq!(
            h.priority_weight,
            99
        );
    }

    assert_eq!(
        p.state(),
        ParserState::FrameLength1
    );
}
