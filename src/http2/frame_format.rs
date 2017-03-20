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

use std::fmt;

use http2::flags::Flags;
use http2::frame_type::FrameType;

/// Frame format.
#[derive(Clone,Copy,PartialEq)]
pub struct FrameFormat {
    flags:                     u8,
    payload_length_frame_type: u32,
    stream_id:                 u32
}

impl FrameFormat {
    /// Create a new `FrameFormat`.
    pub fn new(&mut self, payload_length: u32, frame_type: u8, flags: u8, stream_id: u32)
    -> FrameFormat {
        FrameFormat{
            flags:                     flags,
            payload_length_frame_type: (payload_length << 8) | frame_type as u32,
            stream_id:                 stream_id
        }
   }

   /// Retrieve the frame flags.
   pub fn flags(&self) -> Flags {
       Flags::from_u8(self.flags)
   }

   /// Retrieve the frame type.
   pub fn frame_type(&self) -> FrameType {
       FrameType::from_u8((self.payload_length_frame_type & 0xFF) as u8)
   }

   /// Retrieve the payload length.
   pub fn payload_length(&self) -> u32 {
       self.payload_length_frame_type >> 8
   }

   /// Retrieve the stream identifier.
   pub fn stream_id(&self) -> u32 {
       self.stream_id
   }
}

impl fmt::Display for FrameFormat {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(
            formatter,
            "FrameFormat(flags: {}, frame_type: {}, payload_length: {}, stream_id: {})",
            self.flags(),
            self.frame_type(),
            self.payload_length(),
            self.stream_id
        )
    }
}

impl fmt::Debug for FrameFormat {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(
            formatter,
            "FrameFormat(flags: {}, frame_type: {}, payload_length: {}, stream_id: {})",
            self.flags(),
            self.frame_type(),
            self.payload_length(),
            self.stream_id
        )
    }
}
