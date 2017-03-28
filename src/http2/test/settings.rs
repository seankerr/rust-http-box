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
fn settings() {
    let mut v = Vec::new();

    // frame payload length and type
    pack_u32!(
        v,
        (48 << 8) | 0x4
    );

    // frame frame flags
    pack_u8!(v, 0);

    // frame reserved bit and stream id
    pack_u32!(v, 0x7FFFFFFF);

    // settings identifier
    pack_u16!(
        v,
        0xFFFF
    );

    // settings value
    pack_u32!(
        v,
        0xFFFFFFFF
    );

    let mut h = DebugHandler::new();
    let mut p = Parser::new();

    p.resume(&mut h, &v);

    assert!(Flags::from_u8(h.frame_flags).is_empty());

    assert_eq!(
        FrameType::from_u8(h.frame_type),
        FrameType::Settings
    );

    assert_eq!(
        h.settings_id,
        0xFFFF
    );

    assert_eq!(
        h.settings_value,
        0xFFFFFFFF
    );

    assert_eq!(
        p.state(),
        ParserState::FrameLength1
    );
}
