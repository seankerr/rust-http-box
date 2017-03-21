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

/// Enable push setting.
pub const S_ENABLE_PUSH: u16 = 0x2;

/// Header table size setting.
pub const S_HEADER_TABLE_SIZE: u16 = 0x1;

/// Initial window size setting.
pub const S_INITIAL_WINDOW_SIZE: u16 = 0x4;

/// Maximum concurrent streams setting.
pub const S_MAX_CONCURRENT_STREAMS: u16 = 0x3;

/// Maximum frame size setting.
pub const S_MAX_FRAME_SIZE: u16 = 0x5;

/// Maximum header list size setting.
pub const S_MAX_HEADER_LIST_SIZE: u16 = 0x6;

/// Available settings.
pub enum Setting {
    /// Enable push setting.
    EnablePush(u32),

    /// Header table size setting.
    HeaderTableSize(u32),

    /// Initial window size setting.
    InitialWindowSize(u32),

    /// Maximum concurrent streams setting.
    MaxConcurrentStreams(u32),

    /// Maximum frame size setting.
    MaxFrameSize(u32),

    /// Maximum header list size setting.
    MaxHeaderListSize(u32),

    /// Unsupported setting.
    Unsupported(u32)
}

impl Setting {
    /// Create a new `Setting`.
    pub fn new(&mut self, id: u16, value: u32) -> Setting {
        match id {
            S_HEADER_TABLE_SIZE      => Setting::HeaderTableSize(value),
            S_ENABLE_PUSH            => Setting::EnablePush(value),
            S_MAX_CONCURRENT_STREAMS => Setting::MaxConcurrentStreams(value),
            S_INITIAL_WINDOW_SIZE    => Setting::InitialWindowSize(value),
            S_MAX_FRAME_SIZE         => Setting::MaxFrameSize(value),
            S_MAX_HEADER_LIST_SIZE   => Setting::MaxHeaderListSize(value),
            _                        => Setting::Unsupported(value)
        }
    }

    /// Format this for debug and display purposes.
    fn format(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Setting::EnablePush(x) => {
                write!(
                    formatter,
                    "<Setting::EnablePush: {}>",
                    x
                )
            },
            Setting::HeaderTableSize(x) => {
                write!(
                    formatter,
                    "<Setting::HeaderTableSize: {}>",
                    x
                )
            },
            Setting::InitialWindowSize(x) => {
                write!(
                    formatter,
                    "<Setting::InitialWindowSize: {}>",
                    x
                )
            },
            Setting::MaxConcurrentStreams(x) => {
                write!(
                    formatter,
                    "<Setting::MaxConcurrentStreams: {}>",
                    x
                )
            },
            Setting::MaxFrameSize(x) => {
                write!(
                    formatter,
                    "<Setting::MaxFrameSize: {}>",
                    x
                )
            },
            Setting::MaxHeaderListSize(x) => {
                write!(
                    formatter,
                    "<Setting::MaxHeaderListSize: {}>",
                    x
                )
            },
            Setting::Unsupported(x) => {
                write!(
                    formatter,
                    "<Setting::Unsupported: {}>",
                    x
                )
            }
        }
    }

    /// Indicates that this a `Setting::HeadersTableSize`.
    pub fn is_enable_push(&self) -> bool {
        match *self {
            Setting::EnablePush(_) => true,
            _ => false
        }
    }

    /// Indicates that this a `Setting::HeadersTableSize`.
    pub fn is_header_table_size(&self) -> bool {
        match *self {
            Setting::HeaderTableSize(_) => true,
            _ => false
        }
    }

    /// Indicates that this a `Setting::InitialWindowSize`.
    pub fn is_initial_window_size(&self) -> bool {
        match *self {
            Setting::InitialWindowSize(_) => true,
            _ => false
        }
    }

    /// Indicates that this a `Setting::MaxConcurrentStreams`.
    pub fn is_max_concurrent_streams(&self) -> bool {
        match *self {
            Setting::MaxConcurrentStreams(_) => true,
            _ => false
        }
    }

    /// Indicates that this a `Setting::MaxFrameSize`.
    pub fn is_max_frame_size(&self) -> bool {
        match *self {
            Setting::MaxFrameSize(_) => true,
            _ => false
        }
    }

    /// Indicates that this a `Setting::MaxHeaderListSize`.
    pub fn is_max_header_list_size(&self) -> bool {
        match *self {
            Setting::MaxHeaderListSize(_) => true,
            _ => false
        }
    }

    /// Indicates that this a `Setting::Unsupported`.
    pub fn is_unsupported(&self) -> bool {
        match *self {
            Setting::Unsupported(_) => true,
            _ => false
        }
    }

    /// Retrieve the value.
    pub fn value(&self) -> u32 {
        match *self {
              Setting::EnablePush(x)
            | Setting::HeaderTableSize(x)
            | Setting::InitialWindowSize(x)
            | Setting::MaxConcurrentStreams(x)
            | Setting::MaxFrameSize(x)
            | Setting::MaxHeaderListSize(x)
            | Setting::Unsupported(x) => {
                x
            }
        }
    }
}

impl fmt::Debug for Setting {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        self.format(formatter)
    }
}

impl fmt::Display for Setting {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        self.format(formatter)
    }
}
