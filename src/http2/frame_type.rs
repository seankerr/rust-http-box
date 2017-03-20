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

/// Continuation frame type.
pub const FR_CONTINUATION: u8 = 0x9;

/// Data frame type.
pub const FR_DATA: u8 = 0x0;

/// Go away frame type.
pub const FR_GO_AWAY: u8 = 0x7;

/// Headers frame type.
pub const FR_HEADERS: u8 = 0x1;

/// Ping frame type.
pub const FR_PING: u8 = 0x6;

/// Priority frame type.
pub const FR_PRIORITY: u8 = 0x2;

/// Push promise frame type.
pub const FR_PUSH_PROMISE: u8 = 0x5;

/// Reset stream frame type.
pub const FR_RST_STREAM: u8 = 0x3;

/// Settings frame type.
pub const FR_SETTINGS: u8 = 0x4;

/// Window update frame type.
pub const FR_WINDOW_UPDATE: u8 = 0x8;

/// Unsupported frame type.
pub const FR_UNSUPPORTED: u8 = 0xFF;

/// Frame types.
#[derive(Clone,Copy,PartialEq)]
#[repr(u8)]
pub enum FrameType {
    /// Continuation frame.
    Continuation = FR_CONTINUATION,

    /// Data frame.
    Data = FR_DATA,

    /// Go away frame.
    GoAway = FR_GO_AWAY,

    /// Headers frame.
    Headers = FR_HEADERS,

    /// Ping frame.
    Ping = FR_PING,

    /// Priority frame.
    Priority = FR_PRIORITY,

    /// Push promise frame.
    PushPromise = FR_PUSH_PROMISE,

    /// Reset stream frame.
    RstStream = FR_RST_STREAM,

    /// Settings frame.
    Settings = FR_SETTINGS,

    /// Window update frame.
    WindowUpdate = FR_WINDOW_UPDATE,

    /// Unsupported frame.
    Unsupported = FR_UNSUPPORTED
}

impl FrameType {
    /// Create a new `FrameType` from a `u8`.
    pub fn from_u8(byte: u8) -> FrameType {
        match byte {
            FR_DATA          => FrameType::Data,
            FR_HEADERS       => FrameType::Headers,
            FR_PRIORITY      => FrameType::Priority,
            FR_RST_STREAM    => FrameType::RstStream,
            FR_SETTINGS      => FrameType::Settings,
            FR_PUSH_PROMISE  => FrameType::PushPromise,
            FR_PING          => FrameType::Ping,
            FR_GO_AWAY       => FrameType::GoAway,
            FR_WINDOW_UPDATE => FrameType::WindowUpdate,
            FR_CONTINUATION  => FrameType::Continuation,
            _                => FrameType::Unsupported
        }
    }

    /// Convert this frame type to a byte value.
    pub fn as_byte(&self) -> u8 {
        match *self {
            FrameType::Continuation => FR_CONTINUATION,
            FrameType::Data         => FR_DATA,
            FrameType::GoAway       => FR_GO_AWAY,
            FrameType::Headers      => FR_HEADERS,
            FrameType::Ping         => FR_PING,
            FrameType::PushPromise  => FR_PUSH_PROMISE,
            FrameType::Priority     => FR_PRIORITY,
            FrameType::RstStream    => FR_RST_STREAM,
            FrameType::Settings     => FR_SETTINGS,
            FrameType::WindowUpdate => FR_WINDOW_UPDATE,
            _                       => FR_UNSUPPORTED
        }
    }

    /// Indicates that this is a `FrameType::Continuation`.
    pub fn is_continuation(&self) -> bool {
        match *self {
            FrameType::Continuation => true,
            _ => false
        }
    }

    /// Indicates that this is a `FrameType::Data`.
    pub fn is_data(&self) -> bool {
        match *self {
            FrameType::Data => true,
            _ => false
        }
    }

    /// Indicates that this is a `FrameType::GoAway`.
    pub fn is_go_away(&self) -> bool {
        match *self {
            FrameType::GoAway => true,
            _ => false
        }
    }

    /// Indicates that this is a `FrameType::Headers`.
    pub fn is_headers(&self) -> bool {
        match *self {
            FrameType::Headers => true,
            _ => false
        }
    }

    /// Indicates that this is a `FrameType::Ping`.
    pub fn is_push_ping(&self) -> bool {
        match *self {
            FrameType::Ping => true,
            _ => false
        }
    }

    /// Indicates that this is a `FrameType::Priority`.
    pub fn is_priority(&self) -> bool {
        match *self {
            FrameType::Priority => true,
            _ => false
        }
    }

    /// Indicates that this is a `FrameType::PushPromise`.
    pub fn is_push_promise(&self) -> bool {
        match *self {
            FrameType::PushPromise => true,
            _ => false
        }
    }

    /// Indicates that this is a `FrameType::RstStream`.
    pub fn is_rst_stream(&self) -> bool {
        match *self {
            FrameType::RstStream => true,
            _ => false
        }
    }

    /// Indicates that this is a `FrameType::Settings`.
    pub fn is_settings(&self) -> bool {
        match *self {
            FrameType::Settings => true,
            _ => false
        }
    }

    /// Indicates that this is a `FrameType::Unsupported`.
    pub fn is_unsupported(&self) -> bool {
        match *self {
            FrameType::Unsupported => true,
            _ => false
        }
    }

    /// Indicates that this is a `FrameType::WindowUpdate`.
    pub fn is_window_update(&self) -> bool {
        match *self {
            FrameType::WindowUpdate => true,
            _ => false
        }
    }
}

impl fmt::Debug for FrameType {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            FrameType::Continuation => {
                write!(formatter, "FrameType::Continuation")
            },
            FrameType::Data => {
                write!(formatter, "FrameType::Data")
            },
            FrameType::GoAway => {
                write!(formatter, "FrameType::GoAway")
            },
            FrameType::Headers => {
                write!(formatter, "FrameType::Headers")
            },
            FrameType::Ping => {
                write!(formatter, "FrameType::Ping")
            },
            FrameType::Priority => {
                write!(formatter, "FrameType::Priority")
            },
            FrameType::PushPromise => {
                write!(formatter, "FrameType::PushPromise")
            },
            FrameType::RstStream => {
                write!(formatter, "FrameType::RstStream")
            },
            FrameType::Settings => {
                write!(formatter, "FrameType::Settings")
            },
            FrameType::WindowUpdate => {
                write!(formatter, "FrameType::WindowUpdate")
            },
            FrameType::Unsupported => {
                write!(formatter, "FrameType::Unsupported")
            }
        }
    }
}

impl fmt::Display for FrameType {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            FrameType::Continuation => {
                write!(formatter, "Continuation")
            },
            FrameType::Data => {
                write!(formatter, "Data")
            },
            FrameType::GoAway => {
                write!(formatter, "GoAway")
            },
            FrameType::Headers => {
                write!(formatter, "Headers")
            },
            FrameType::Ping => {
                write!(formatter, "Ping")
            },
            FrameType::Priority => {
                write!(formatter, "Priority")
            },
            FrameType::PushPromise => {
                write!(formatter, "PushPromise")
            },
            FrameType::RstStream => {
                write!(formatter, "RstStream")
            },
            FrameType::Settings => {
                write!(formatter, "Settings")
            },
            FrameType::WindowUpdate => {
                write!(formatter, "WindowUpdate")
            },
            FrameType::Unsupported => {
                write!(formatter, "Unsupported")
            }
        }
    }
}
