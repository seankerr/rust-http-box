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

/// Acknowledgement flag.
pub const FL_ACK: u8 = 0x1;

/// End headers flag.
pub const FL_END_HEADERS: u8 = 0x4;

/// End stream flag.
pub const FL_END_STREAM: u8 = 0x1;

/// Padded flag.
pub const FL_PADDED: u8 = 0x8;

/// Priority flag.
pub const FL_PRIORITY: u8 = 0x20;

/// Flags.
#[derive(Clone,Copy,PartialEq)]
pub struct Flags {
    flags: u8
}

/// Flags.
impl Flags {
    /// Create a new `Flags` from a `u8`.
    pub fn from_u8(byte: u8) -> Flags {
        Flags {
            flags: byte
        }
    }

    /// Convert this flags to its byte value.
    pub fn as_byte(&self) -> u8 {
        self.flags
    }

    /// Indicates that the ack flag has been set.
    pub fn is_ack(&self) -> bool {
        self.flags & FL_ACK == FL_ACK
    }

    /// Indicates that the flags are empty.
    pub fn is_empty(&self) -> bool {
        self.flags == 0
    }

    /// Indicates that the end headers flag has been set.
    pub fn is_end_headers(&self) -> bool {
        self.flags & FL_END_HEADERS == FL_END_HEADERS
    }

    /// Indicates that the end stream flag has been set.
    pub fn is_end_stream(&self) -> bool {
        self.flags & FL_END_STREAM == FL_END_STREAM
    }

    /// Indicates that the padded flag has been set.
    pub fn is_padded(&self) -> bool {
        self.flags & FL_PADDED == FL_PADDED
    }

    /// Indicates that the priority flag has been set.
    pub fn is_priority(&self) -> bool {
        self.flags & FL_PRIORITY == FL_PRIORITY
    }
}

impl fmt::Debug for Flags {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(
            formatter,
            "<Flags: ack: {}, end_headers: {}, end_stream: {}, padded: {}, priority: {}>",
            self.flags & FL_ACK == FL_ACK,
            self.flags & FL_END_HEADERS == FL_END_HEADERS,
            self.flags & FL_END_STREAM == FL_END_STREAM,
            self.flags & FL_PADDED == FL_PADDED,
            self.flags & FL_PRIORITY == FL_PRIORITY
        )
    }
}

impl fmt::Display for Flags {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(
            formatter,
            "<Flags: ack: {}, end_headers: {}, end_stream: {}, padded: {}, priority: {}>",
            self.flags & FL_ACK == FL_ACK,
            self.flags & FL_END_HEADERS == FL_END_HEADERS,
            self.flags & FL_END_STREAM == FL_END_STREAM,
            self.flags & FL_PADDED == FL_PADDED,
            self.flags & FL_PRIORITY == FL_PRIORITY
        )
    }
}
