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

//! HTTP 2.x parser, states, and errors.

#![allow(dead_code)]

use fsm::{ ParserValue,
           Success };

use byte_slice::ByteStream;
use std::{ fmt,
           u16,
           u32 };

// -------------------------------------------------------------------------------------------------
// ERROR CODES
// -------------------------------------------------------------------------------------------------

/// Cancel error code.
const E_CANCEL: u8 = 0x8;

/// Compression error code.
const E_COMPRESSION: u8 = 0x9;

/// Connect error code.
const E_CONNECT: u8 = 0xA;

/// Enhance your calm error code.
const E_ENHANCE_YOUR_CALM: u8 = 0xB;

/// Flow control error code.
const E_FLOW_CONTROL: u8 = 0x3;

/// Frame size error code.
const E_FRAME_SIZE: u8 = 0x6;

/// HTTP/1.1 required error code.
const E_HTTP_1_1_REQUIRED: u8 = 0xD;

/// Inadequate security error code.
const E_INADEQUATE_SECURITY: u8 = 0xC;

/// No error code.
const E_NO_ERROR: u8 = 0x0;

/// Internal error code.
const E_INTERNAL: u8 = 0x2;

/// Protocol error code.
const E_PROTOCOL: u8 = 0x1;

/// Refused stream error code.
const E_REFUSED_STREAM: u8 = 0x7;

/// Settings timeout error code.
const E_SETTINGS_TIMEOUT: u8 = 0x4;

/// Stream closed error code.
const E_STREAM_CLOSED: u8 = 0x5;

// -------------------------------------------------------------------------------------------------
// FLAGS
// -------------------------------------------------------------------------------------------------

/// End headers flag.
const FL_END_HEADERS: u8 = 0x4;

/// End stream flag.
const FL_END_STREAM: u8 = 0x1;

/// Padded flag.
const FL_PADDED: u8 = 0x8;

/// Priority flag.
const FL_PRIORITY: u8 = 0x20;

// -------------------------------------------------------------------------------------------------
// FRAME TYPES
// -------------------------------------------------------------------------------------------------

/// Continuation frame type.
const FR_CONTINUATION: u8 = 0x9;

/// Data frame type.
const FR_DATA: u8 = 0x0;

/// Go away frame type.
const FR_GO_AWAY: u8 = 0x7;

/// Headers frame type.
const FR_HEADERS: u8 = 0x1;

/// Ping frame type.
const FR_PING: u8 = 0x6;

/// Priority frame type.
const FR_PRIORITY: u8 = 0x2;

/// Push promise frame type.
const FR_PUSH_PROMISE: u8 = 0x5;

/// Reset stream frame type.
const FR_RST_STREAM: u8 = 0x3;

/// Settings frame type.
const FR_SETTINGS: u8 = 0x4;

/// Window update frame type.
const FR_WINDOW_UPDATE: u8 = 0x8;

// -------------------------------------------------------------------------------------------------
// SETTINGS
// -------------------------------------------------------------------------------------------------

/// Enable push setting.
const S_ENABLE_PUSH: u16 = 0x2;

/// Header table size setting.
const S_HEADER_TABLE_SIZE: u16 = 0x1;

/// Initial window size setting.
const S_INITIAL_WINDOW_SIZE: u16 = 0x4;

/// Maximum concurrent streams setting.
const S_MAX_CONCURRENT_STREAMS: u16 = 0x3;

/// Maximum frame size setting.
const S_MAX_FRAME_SIZE: u16 = 0x5;

/// Maximum header list size setting.
const S_MAX_HEADER_LIST_SIZE: u16 = 0x6;

// -------------------------------------------------------------------------------------------------
// MACROS
// -------------------------------------------------------------------------------------------------

/// Handle data with padding state.
macro_rules! data_with_padding {
    ($parser:expr, $context:expr, $callback:ident) => ({
        if bs_available!($context) >= (($parser.length_flags & 0xFFFFFF) as usize)
                                      - $parser.bit_data2 as usize {
            // collect remaining data
            bs_jump!($context, ($parser.length_flags & 0xFFFFFF) as usize
                               - $parser.bit_data2 as usize);

            set_state!($parser, FrameLength1, frame_length1);

            if $parser.handler.$callback(bs_slice!($context), true) {
                transition!($parser, $context);
            } else {
                exit_callback!($parser, $context);
            }
        } else {
            // collect remaining slice
            bs_jump!($context, bs_available!($context));

            if $parser.handler.$callback(bs_slice!($context), false) {
                exit_eos!($parser, $context);
            } else {
                exit_callback!($parser, $context);
            }
        }
    });
}

/// Handle data without padding state.
macro_rules! data_without_padding {
    ($parser:expr, $context:expr, $callback:ident) => ({
        if bs_available!($context) >= ($parser.length_flags & 0xFFFFFF) as usize {
            // collect remaining data
            bs_jump!($context, ($parser.length_flags & 0xFFFFFF) as usize);

            set_state!($parser, FrameLength1, frame_length1);

            if $parser.handler.$callback(bs_slice!($context), true) {
                transition!($parser, $context);
            } else {
                exit_callback!($parser, $context);
            }
        } else {
            // collect remaining slice
            bs_jump!($context, bs_available!($context));

            if $parser.handler.$callback(bs_slice!($context), false) {
                exit_eos!($parser, $context);
            } else {
                exit_callback!($parser, $context);
            }
        }
    });
}

/// Retrieve a u8.
macro_rules! get_u8 {
    ($context:expr) => ({
        $context.stream_index += 1;

        $context.stream[$context.stream_index - 1]
    });
}

/// Indicates that a flag is set.
macro_rules! has_flag {
    ($parser:expr, $flag:expr) => (
        ($parser.length_flags >> 24) as u8 & $flag == $flag
    );
}

/// Read a u16.
macro_rules! read_u16 {
    ($context:expr, $into:expr) => ({
        $into |= ($context.stream[$context.stream_index] as u16) << 8;
        $into |= $context.stream[$context.stream_index + 2] as u16;

        bs_jump!($context, 2);
    });
}

/// Read a u32.
macro_rules! read_u32 {
    ($context:expr, $into:expr) => ({
        $into |= ($context.stream[$context.stream_index] as u32) << 24;
        $into |= ($context.stream[$context.stream_index + 1] as u32) << 16;
        $into |= ($context.stream[$context.stream_index + 2] as u32) << 8;
        $into |= $context.stream[$context.stream_index + 3] as u32;

        bs_jump!($context, 4);
    });
}

// -------------------------------------------------------------------------------------------------

/// Error codes.
#[repr(u8)]
pub enum ErrorCode {
    /// Cancel error.
    Cancel = E_CANCEL,

    /// Compression error.
    Compression = E_COMPRESSION,

    /// Connect error.
    Connect = E_CONNECT,

    /// Enhance your calm error.
    EnhanceYourCalm = E_ENHANCE_YOUR_CALM,

    /// Flow control error.
    FlowControl = E_FLOW_CONTROL,

    /// Frame size error.
    FrameSize = E_FRAME_SIZE,

    /// HTTP/1.1 required error.
    Http11Required = E_HTTP_1_1_REQUIRED,

    /// Inadequate security error.
    InadequateSecurity = E_INADEQUATE_SECURITY,

    /// No error.
    NoError = E_NO_ERROR,

    /// Internal error.
    Internal = E_INTERNAL,

    /// Protocol error.
    Protocol = E_PROTOCOL,

    /// Refused stream error.
    RefusedStream = E_REFUSED_STREAM,

    /// Settings timeout error.
    SettingsTimeout = E_SETTINGS_TIMEOUT,

    /// Stream closed error.
    StreamClosed = E_STREAM_CLOSED
}

impl ErrorCode {
    /// Create a new `ErrorCode` from a `u8`.
    pub fn from_u8(byte: u8) -> Option<ErrorCode> {
        match byte {
            E_CANCEL              => Some(ErrorCode::Cancel),
            E_COMPRESSION         => Some(ErrorCode::Compression),
            E_CONNECT             => Some(ErrorCode::Connect),
            E_ENHANCE_YOUR_CALM   => Some(ErrorCode::EnhanceYourCalm),
            E_FLOW_CONTROL        => Some(ErrorCode::FlowControl),
            E_FRAME_SIZE          => Some(ErrorCode::FrameSize),
            E_HTTP_1_1_REQUIRED   => Some(ErrorCode::Http11Required),
            E_INADEQUATE_SECURITY => Some(ErrorCode::InadequateSecurity),
            E_INTERNAL            => Some(ErrorCode::Internal),
            E_NO_ERROR            => Some(ErrorCode::NoError),
            E_PROTOCOL            => Some(ErrorCode::Protocol),
            E_REFUSED_STREAM      => Some(ErrorCode::RefusedStream),
            E_SETTINGS_TIMEOUT    => Some(ErrorCode::SettingsTimeout),
            E_STREAM_CLOSED       => Some(ErrorCode::StreamClosed),
            _                     => None
        }
    }
}

// -------------------------------------------------------------------------------------------------

/// Flags.
#[repr(u8)]
pub enum Flag {
    /// End headers flag.
    EndHeaders = FL_END_HEADERS,

    /// End stream flag.
    EndStream = FL_END_STREAM,

    /// Padded flag.
    Padded = FL_PADDED,

    /// Priority flag.
    Priority = FL_PRIORITY
}

impl Flag {
    /// Create a new `Flag` from a `u8`.
    pub fn from_u8(byte: u8) -> Option<Flag> {
        match byte {
            FL_END_STREAM  => Some(Flag::EndStream),
            FL_END_HEADERS => Some(Flag::EndHeaders),
            FL_PADDED      => Some(Flag::Padded),
            FL_PRIORITY    => Some(Flag::Priority),
            _              => None
        }
    }
}

impl fmt::Debug for Flag {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Flag::EndHeaders => {
                write!(formatter, "Flag::EndHeaders")
            }
            Flag::EndStream => {
                write!(formatter, "Flag::EndStream")
            },
            Flag::Padded => {
                write!(formatter, "Flag::Padded")
            },
            Flag::Priority => {
                write!(formatter, "Flag::Priority")
            }
        }
    }
}

impl fmt::Display for Flag {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Flag::EndHeaders => {
                write!(formatter, "EndHeaders")
            }
            Flag::EndStream => {
                write!(formatter, "EndStream")
            },
            Flag::Padded => {
                write!(formatter, "Padded")
            },
            Flag::Priority => {
                write!(formatter, "Priority")
            }
        }
    }
}

// -------------------------------------------------------------------------------------------------

/// Frame types.
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
    WindowUpdate = FR_WINDOW_UPDATE
}

impl FrameType {
    /// Create a new `FrameType` from a `u8`.
    pub fn from_u8(byte: u8) -> Option<FrameType> {
        match byte {
            FR_DATA          => Some(FrameType::Data),
            FR_HEADERS       => Some(FrameType::Headers),
            FR_PRIORITY      => Some(FrameType::Priority),
            FR_RST_STREAM    => Some(FrameType::RstStream),
            FR_SETTINGS      => Some(FrameType::Settings),
            FR_PUSH_PROMISE  => Some(FrameType::PushPromise),
            FR_PING          => Some(FrameType::Ping),
            FR_GO_AWAY       => Some(FrameType::GoAway),
            FR_WINDOW_UPDATE => Some(FrameType::WindowUpdate),
            FR_CONTINUATION  => Some(FrameType::Continuation),
            _                => None
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
            }
        }
    }
}

// -------------------------------------------------------------------------------------------------

/// Type that handles HTTP/2.x parser events.
#[allow(unused_variables)]
pub trait HttpHandler {
    /// Callback that is executed when data frame data has been located.
    ///
    /// *Note:* This may be executed multiple times in order to supply the entire segment.
    ///
    /// **Arguments:**
    ///
    /// **`data`**
    ///
    /// The data.
    ///
    /// **`finished`**
    ///
    /// Indicates this is the last chunk of the data.
    ///
    /// **Returns:**
    ///
    /// `true` when parsing should continue, `false` to exit the parser function prematurely with
    /// [`Success::Callback`](../fsm/enum.Success.html#variant.Callback).
    fn on_data_data(&mut self, data: &[u8], finished: bool) -> bool {
        true
    }

    /// Callback that is executed when a new frame has been located.
    ///
    /// **Arguments:**
    ///
    /// **`length`**
    ///
    /// The length.
    ///
    /// **`frame_type`**
    ///
    /// The type.
    ///
    /// **`flags`**
    ///
    /// The flags.
    ///
    /// **`reserved`**
    ///
    /// Indicates the reserved bit was set.
    ///
    /// **`stream_id`**
    ///
    /// The stream identifier.
    ///
    /// **Returns:**
    ///
    /// `true` when parsing should continue, `false` to exit the parser function prematurely with
    /// [`Success::Callback`](../fsm/enum.Success.html#variant.Callback).
    fn on_frame(&mut self, length: u32, frame_type: u8, flags: u8, reserved: bool,
                stream_id: u32) -> bool {
        true
    }

    /// Callback that is executed when go away frame debug data has been located.
    ///
    /// *Note:* This may be executed multiple times in order to supply the entire segment.
    ///
    /// **Arguments:**
    ///
    /// **`data`**
    ///
    /// The data.
    ///
    /// **`finished`**
    ///
    /// Indicates this is the last chunk of the data.
    ///
    /// **Returns:**
    ///
    /// `true` when parsing should continue, `false` to exit the parser function prematurely with
    /// [`Success::Callback`](../fsm/enum.Success.html#variant.Callback).
    fn on_go_away_debug_data(&mut self, data: &[u8], finished: bool) -> bool {
        true
    }

    /// Callback that is executed when a go away frame error code has been located.
    ///
    /// **Arguments:**
    ///
    /// **`error_code`**
    ///
    /// The error code.
    ///
    /// **Returns:**
    ///
    /// `true` when parsing should continue, `false` to exit the parser function prematurely with
    /// [`Success::Callback`](../fsm/enum.Success.html#variant.Callback).
    fn on_go_away_error_code(&mut self, error_code: u32) -> bool {
        true
    }

    /// Callback that is executed when a go away frame stream identifier has been located.
    ///
    /// **Arguments:**
    ///
    /// **`reserved`**
    ///
    /// Indicates the reserved bit was set.
    ///
    /// **`stream_id`**
    ///
    /// The stream identifier.
    ///
    /// **Returns:**
    ///
    /// `true` when parsing should continue, `false` to exit the parser function prematurely with
    /// [`Success::Callback`](../fsm/enum.Success.html#variant.Callback).
    fn on_go_away_stream_id(&mut self, reserved: bool, stream_id: u32) -> bool {
        true
    }

    /// Callback that is executed when a headers frame fragment has been located.
    ///
    /// *Note:* This may be executed multiple times in order to supply the entire segment.
    ///
    /// **Arguments:**
    ///
    /// **`fragment`**
    ///
    /// The fragment.
    ///
    /// **`finished`**
    ///
    /// Indicates this is the last chunk of the fragment.
    ///
    /// **Returns:**
    ///
    /// `true` when parsing should continue, `false` to exit the parser function prematurely with
    /// [`Success::Callback`](../fsm/enum.Success.html#variant.Callback).
    fn on_headers_fragment(&mut self, fragment: &[u8], finished: bool) -> bool {
        true
    }

    /// Callback that is executed when ping frame data has been located.
    ///
    /// *Note:* This may be executed multiple times in order to supply the entire segment.
    ///
    /// **Arguments:**
    ///
    /// **`data`**
    ///
    /// The data.
    ///
    /// **`finished`**
    ///
    /// Indicates this is the last chunk of the data.
    ///
    /// **Returns:**
    ///
    /// `true` when parsing should continue, `false` to exit the parser function prematurely with
    /// [`Success::Callback`](../fsm/enum.Success.html#variant.Callback).
    fn on_ping_data(&mut self, data: &[u8], finished: bool) -> bool {
        true
    }

    /// Callback that is executed when a priority frame stream identifier has been located.
    ///
    /// **Arguments:**
    ///
    /// **`exclusive`**
    ///
    /// Indicates the stream identifier is exclusive.
    ///
    /// **`stream_id`**
    ///
    /// The stream identifier.
    ///
    /// **Returns:**
    ///
    /// `true` when parsing should continue, `false` to exit the parser function prematurely with
    /// [`Success::Callback`](../fsm/enum.Success.html#variant.Callback).
    fn on_priority_stream_id(&mut self, exclusive: bool, stream_id: u32) -> bool {
        true
    }

    /// Callback that is executed when a priority frame weight has been located.
    ///
    /// *Note:* This may be executed multiple times in order to supply the entire segment.
    ///
    /// **Arguments:**
    ///
    /// **`weight`**
    ///
    /// The priority weight.
    ///
    /// **Returns:**
    ///
    /// `true` when parsing should continue, `false` to exit the parser function prematurely with
    /// [`Success::Callback`](../fsm/enum.Success.html#variant.Callback).
    fn on_priority_weight(&mut self, weight: u8) -> bool {
        true
    }

    /// Callback that is executed when a push promise frame stream identifier has been located.
    ///
    /// **Arguments:**
    ///
    /// **`reserved`**
    ///
    /// Indicates the reserved bit was set.
    ///
    /// **`stream_id`**
    ///
    /// The stream identifier.
    ///
    /// **Returns:**
    ///
    /// `true` when parsing should continue, `false` to exit the parser function prematurely with
    /// [`Success::Callback`](../fsm/enum.Success.html#variant.Callback).
    fn on_push_promise_stream_id(&mut self, reserved: bool, stream_id: u32) -> bool {
        true
    }

    /// Callback that is executed when a rst stream frame error code has been located.
    ///
    /// **Arguments:**
    ///
    /// **`error_code`**
    ///
    /// The error code.
    ///
    /// **Returns:**
    ///
    /// `true` when parsing should continue, `false` to exit the parser function prematurely with
    /// [`Success::Callback`](../fsm/enum.Success.html#variant.Callback).
    fn on_rst_stream_error_code(&mut self, error_code: u32) -> bool {
        true
    }

    /// Callback that is executed when a settings frame identifier has been located.
    ///
    /// **Arguments:**
    ///
    /// **`id`**
    ///
    /// The setting identifier.
    ///
    /// **Returns:**
    ///
    /// `true` when parsing should continue, `false` to exit the parser function prematurely with
    /// [`Success::Callback`](../fsm/enum.Success.html#variant.Callback).
    fn on_settings_id(&mut self, id: u16) -> bool {
        true
    }

    /// Callback that is executed when a settings frame value has been located.
    ///
    /// **Arguments:**
    ///
    /// **`value`**
    ///
    /// The setting value.
    ///
    /// **Returns:**
    ///
    /// `true` when parsing should continue, `false` to exit the parser function prematurely with
    /// [`Success::Callback`](../fsm/enum.Success.html#variant.Callback).
    fn on_settings_value(&mut self, value: u32) -> bool {
        true
    }

    /// Callback that is executed when unknown frame data has been located.
    ///
    /// *Note:* This may be executed multiple times in order to supply the entire segment.
    ///
    /// **Arguments:**
    ///
    /// **`data`**
    ///
    /// The data.
    ///
    /// **`finished`**
    ///
    /// Indicates this is the last chunk of the data.
    ///
    /// **Returns:**
    ///
    /// `true` when parsing should continue, `false` to exit the parser function prematurely with
    /// [`Success::Callback`](../fsm/enum.Success.html#variant.Callback).
    fn on_unknown_data(&mut self, data: &[u8], finished: bool) -> bool {
        true
    }

    /// Callback that is executed when a window update frame size increment has been located.
    ///
    /// *Note:* This may be executed multiple times in order to supply the entire segment.
    ///
    /// **Arguments:**
    ///
    /// **`size_increment`**
    ///
    /// The size increment.
    ///
    /// **Returns:**
    ///
    /// `true` when parsing should continue, `false` to exit the parser function prematurely with
    /// [`Success::Callback`](../fsm/enum.Success.html#variant.Callback).
    fn on_window_update_size_increment(&mut self, size_increment: u32) -> bool {
        true
    }
}

// -------------------------------------------------------------------------------------------------

/// Parser error messages.
#[derive(Clone,Copy,PartialEq)]
pub enum ParserError {
    /// Parsing has failed.
    Dead,
/*
    /// Invalid frame flags on byte `u8`.
    FrameFlags(u8),

    /// Invalid frame length on byte `u8`.
    FrameLength(u8),

    /// Invalid frame type on byte `u8`.
    FrameType(u8),

    /// Invalid frame stream identifier on byte `u8`.
    FrameStreamId(u8),

    /// Invalid headers dependency stream identifier on byte `u8`.
    HeadersStreamId(u8),

    /// Invalid headers weight on byte `u8`.
    HeadersWeight(u8),

    /// Invalid priority dependency stream identifier on byte `u8`.
    PriorityStreamId(u8),
*/
}

impl fmt::Debug for ParserError {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            ParserError::Dead => {
                write!(formatter, "ParserError::Dead(Parser is dead)")
            },
        }
    }
}

impl fmt::Display for ParserError {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            ParserError::Dead => {
                write!(formatter, "Parser is dead")
            },
        }
    }
}

// -------------------------------------------------------------------------------------------------

/// Parser states.
#[derive(Clone,Copy,Debug,PartialEq)]
#[repr(u8)]
pub enum ParserState {
    /// An error was returned from a call to `Parser::parse()`.
    Dead,

    // ---------------------------------------------------------------------------------------------
    // FRAME STATES
    // ---------------------------------------------------------------------------------------------

    /// Parsing frame length first byte.
    FrameLength1,

    /// Parsing frame length second byte.
    FrameLength2,

    /// Parsing frame length third byte.
    FrameLength3,

    /// Parsing frame type.
    FrameType,

    /// Parsing frame flags.
    FrameFlags,

    /// Parsing frame stream identifier first byte.
    FrameStreamId1,

    /// Parsing frame stream identifier second byte.
    FrameStreamId2,

    /// Parsing frame stream identifier third byte.
    FrameStreamId3,

    /// Parsing frame stream identifier fourth byte.
    FrameStreamId4,

    /// Frame parsing finished.
    FrameEnd,

    /// Parsing end-of-frame padding.
    FramePadding,

    // ---------------------------------------------------------------------------------------------
    // DATA STATES
    // ---------------------------------------------------------------------------------------------

    /// Parsing data pad length.
    DataPadLength,

    /// Parsing data without padding.
    DataDataWithoutPadding,

    /// Parsing data with padding.
    DataDataWithPadding,

    // ---------------------------------------------------------------------------------------------
    // GO AWAY STATES
    // ---------------------------------------------------------------------------------------------

    /// Parsing go away stream identifier first byte.
    GoAwayStreamId1,

    /// Parsing go away stream identifier second byte.
    GoAwayStreamId2,

    /// Parsing go away stream identifier third byte.
    GoAwayStreamId3,

    /// Parsing go away stream identifier fourth byte.
    GoAwayStreamId4,

    /// Parsing go away error code first byte.
    GoAwayErrorCode1,

    /// Parsing go away error code second byte.
    GoAwayErrorCode2,

    /// Parsing go away error code third byte.
    GoAwayErrorCode3,

    /// Parsing go away error code fourth byte.
    GoAwayErrorCode4,

    /// Parsing go away debug data.
    GoAwayDebugData,

    // ---------------------------------------------------------------------------------------------
    // HEADERS STATES
    // ---------------------------------------------------------------------------------------------

    /// Parsing headers pad length.
    HeadersPadLength,

    /// Parsing headers stream identifier first byte.
    HeadersStreamId1,

    /// Parsing headers stream identifier second byte.
    HeadersStreamId2,

    /// Parsing headers stream identifier third byte.
    HeadersStreamId3,

    /// Parsing headers stream identifier fourth byte.
    HeadersStreamId4,

    /// Parsing headers weight.
    HeadersWeight,

    /// Parsing headers fragment without padding.
    HeadersFragmentWithoutPadding,

    /// Parsing headers fragment with padding.
    HeadersFragmentWithPadding,

    // ---------------------------------------------------------------------------------------------
    // PING STATES
    // ---------------------------------------------------------------------------------------------

    /// Parsing ping data.
    PingData,

    // ---------------------------------------------------------------------------------------------
    // PRIORITY STATES
    // ---------------------------------------------------------------------------------------------

    /// Parsing priority stream identifier first byte.
    PriorityStreamId1,

    /// Parsing priority stream identifier second byte.
    PriorityStreamId2,

    /// Parsing priority stream identifier third byte.
    PriorityStreamId3,

    /// Parsing priority stream identifier fourth byte.
    PriorityStreamId4,

    /// Parsing priority weight.
    PriorityWeight,

    // ---------------------------------------------------------------------------------------------
    // PUSH PROMISE STATES
    // ---------------------------------------------------------------------------------------------

    /// Parsing push promise pad length.
    PushPromisePadLength,

    /// Parsing push promise stream identifier first byte.
    PushPromiseStreamId1,

    /// Parsing push promise stream identifier second byte.
    PushPromiseStreamId2,

    /// Parsing push promise stream identifier third byte.
    PushPromiseStreamId3,

    /// Parsing push promise stream identifier fourth byte.
    PushPromiseStreamId4,

    // ---------------------------------------------------------------------------------------------
    // RST STREAM STATES
    // ---------------------------------------------------------------------------------------------

    /// Parsing rst stream error code first byte.
    RstStreamErrorCode1,

    /// Parsing rst stream error code second byte.
    RstStreamErrorCode2,

    /// Parsing rst stream error code third byte.
    RstStreamErrorCode3,

    /// Parsing rst stream error code fourth byte.
    RstStreamErrorCode4,

    // ---------------------------------------------------------------------------------------------
    // SETTINGS STATES
    // ---------------------------------------------------------------------------------------------

    /// Parsing settings identifier first byte.
    SettingsId1,

    /// Parsing settings identifier second byte.
    SettingsId2,

    /// Parsing settings value first byte.
    SettingsValue1,

    /// Parsing settings value second byte.
    SettingsValue2,

    /// Parsing settings value third byte.
    SettingsValue3,

    /// Parsing settings value fourth byte.
    SettingsValue4,

    // ---------------------------------------------------------------------------------------------
    // UNKNOWN STATES
    // ---------------------------------------------------------------------------------------------

    /// Parsing unknown pad length.
    UnknownPadLength,

    /// Parsing unknown data without padding.
    UnknownDataWithoutPadding,

    /// Parsing unknown data with padding.
    UnknownDataWithPadding,

    // ---------------------------------------------------------------------------------------------
    // WINDOW UPDATE STATES
    // ---------------------------------------------------------------------------------------------

    /// Parsing window update increment first byte.
    WindowUpdateIncrement1,

    /// Parsing window update increment second byte.
    WindowUpdateIncrement2,

    /// Parsing window update increment third byte.
    WindowUpdateIncrement3,

    /// Parsing window update increment fourth byte.
    WindowUpdateIncrement4
}

// -------------------------------------------------------------------------------------------------

/// Settings.
#[repr(u16)]
pub enum Setting {
    /// Enable push setting.
    EnablePush = S_ENABLE_PUSH,

    /// Header table size setting.
    HeaderTableSize = S_HEADER_TABLE_SIZE,

    /// Initial window size setting.
    InitialWindowSize = S_INITIAL_WINDOW_SIZE,

    /// Maximum concurrent streams setting.
    MaxConcurrentStreams = S_MAX_CONCURRENT_STREAMS,

    /// Maximum frame size setting.
    MaxFrameSize = S_MAX_FRAME_SIZE,

    /// Maximum header list size setting.
    MaxHeaderListSize = S_MAX_HEADER_LIST_SIZE
}

impl Setting {
    /// Create a new `Setting` from `u16`.
    pub fn from_u16(bytes: u16) -> Option<Setting> {
        match bytes {
            S_HEADER_TABLE_SIZE      => Some(Setting::HeaderTableSize),
            S_ENABLE_PUSH            => Some(Setting::EnablePush),
            S_MAX_CONCURRENT_STREAMS => Some(Setting::MaxConcurrentStreams),
            S_INITIAL_WINDOW_SIZE    => Some(Setting::InitialWindowSize),
            S_MAX_FRAME_SIZE         => Some(Setting::MaxFrameSize),
            S_MAX_HEADER_LIST_SIZE   => Some(Setting::MaxHeaderListSize),
            _                        => None
        }
    }
}

impl fmt::Debug for Setting {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Setting::EnablePush => {
                write!(formatter, "Setting::EnablePush")
            },
            Setting::HeaderTableSize => {
                write!(formatter, "Setting::HeaderTableSize")
            },
            Setting::InitialWindowSize => {
                write!(formatter, "Setting::InitialWindowSize")
            },
            Setting::MaxConcurrentStreams => {
                write!(formatter, "Setting::MaxConcurrentStreams")
            },
            Setting::MaxFrameSize => {
                write!(formatter, "Setting::MaxFrameSize")
            },
            Setting::MaxHeaderListSize => {
                write!(formatter, "Setting::MaxHeaderListSize")
            }
        }
    }
}

impl fmt::Display for Setting {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Setting::EnablePush => {
                write!(formatter, "EnablePush")
            },
            Setting::HeaderTableSize => {
                write!(formatter, "HeaderTableSize")
            },
            Setting::InitialWindowSize => {
                write!(formatter, "InitialWindowSize")
            },
            Setting::MaxConcurrentStreams => {
                write!(formatter, "MaxConcurrentStreams")
            },
            Setting::MaxFrameSize => {
                write!(formatter, "MaxFrameSize")
            },
            Setting::MaxHeaderListSize => {
                write!(formatter, "MaxHeaderListSize")
            }
        }
    }
}

// -------------------------------------------------------------------------------------------------

/// HTTP 2.x parser.
pub struct Parser<'a, T: HttpHandler> {
    /// Bit data that stores parser state details.
    bit_data1: u32,
    bit_data2: u8,

    /// Total byte count processed.
    byte_count: usize,

    /// Handler implementation.
    handler: T,

    /// Frame length and flags.
    length_flags: u32,

    /// Current state.
    state: ParserState,

    /// Current state function.
    state_function: fn(&mut Parser<'a, T>, &mut ByteStream) -> Result<ParserValue, ParserError>
}

impl<'a, T: HttpHandler> Parser<'a, T> {
    /// Create a new `Parser`.
    ///
    /// # Arguments
    ///
    /// **`handler`**
    ///
    /// The handler implementation.
    fn new(handler: T) -> Parser<'a, T> {
        Parser{ bit_data1:      0,
                bit_data2:      0,
                byte_count:     0,
                handler:        handler,
                length_flags:   0,
                state:          ParserState::FrameLength1,
                state_function: Parser::frame_length1 }
    }

    /// Retrieve the total byte count processed since the instantiation of `Parser`.
    ///
    /// The byte count is updated when `resume()` completes. This means that if a
    /// call to `byte_count()` is executed from within a callback, it will be accurate within
    /// `stream.len()` bytes. For precise accuracy, the best time to retrieve the byte count is
    /// outside of all callbacks.
    pub fn byte_count(&self) -> usize {
        self.byte_count
    }

    /// Retrieve the handler implementation.
    pub fn handler(&mut self) -> &mut T {
        &mut self.handler
    }

    /// Reset `Parser` to its initial state.
    pub fn reset(&mut self) {
        self.bit_data1       = 0;
        self.bit_data2       = 0;
        self.byte_count      = 0;
        self.length_flags    = 0;
        self.state           = ParserState::FrameLength1;
        self.state_function  = Parser::frame_length1;
    }

    // ---------------------------------------------------------------------------------------------
    // FRAME STATES
    // ---------------------------------------------------------------------------------------------

    #[inline]
    fn frame_length1(&mut self, context: &mut ByteStream)
    -> Result<ParserValue, ParserError> {
        exit_if_eos!(self, context);

        // reset frame details
        self.bit_data1    = 0;
        self.bit_data2    = 0;
        self.length_flags = 0;

        if bs_available!(context) >= 3 {
            self.length_flags |= (context.stream[context.stream_index]  as u32) << 16;
            self.length_flags |= (context.stream[context.stream_index + 1] as u32) << 8;
            self.length_flags |= context.stream[context.stream_index + 2] as u32;
            self.length_flags  = u32::from_be(self.length_flags);

            bs_jump!(context, 3);
        } else {
            self.length_flags = get_u8!(context) as u32;
        }

        transition_fast!(self, context,
                         FrameLength2, frame_length2);
    }

    #[inline]
    fn frame_length2(&mut self, context: &mut ByteStream)
    -> Result<ParserValue, ParserError> {
        exit_if_eos!(self, context);

        self.length_flags <<= 8;
        self.length_flags  |= get_u8!(context) as u32;

        transition_fast!(self, context,
                         FrameLength3, frame_length3);
    }

    #[inline]
    fn frame_length3(&mut self, context: &mut ByteStream)
    -> Result<ParserValue, ParserError> {
        exit_if_eos!(self, context);

        self.length_flags <<= 8;
        self.length_flags  |= get_u8!(context) as u32;
        self.length_flags   = u32::from_be(self.length_flags);

        transition_fast!(self, context,
                         FrameType, frame_type);
    }

    #[inline]
    fn frame_type(&mut self, context: &mut ByteStream)
    -> Result<ParserValue, ParserError> {
        exit_if_eos!(self, context);

        self.bit_data2 = get_u8!(context);

        transition_fast!(self, context,
                         FrameFlags, frame_flags);
    }

    #[inline]
    fn frame_flags(&mut self, context: &mut ByteStream)
    -> Result<ParserValue, ParserError> {
        exit_if_eos!(self, context);

        self.length_flags |= (get_u8!(context) as u32) << 24;

        transition_fast!(self, context,
                         FrameStreamId1, frame_stream_id1);
    }

    #[inline]
    fn frame_stream_id1(&mut self, context: &mut ByteStream)
    -> Result<ParserValue, ParserError> {
        exit_if_eos!(self, context);

        if bs_available!(context) >= 4 {
            read_u32!(context, self.bit_data1);

            self.bit_data1 = u32::from_be(self.bit_data1);

            transition_fast!(self, context,
                             FrameEnd, frame_end);
        } else {
            self.bit_data1 = get_u8!(context) as u32;

            transition_fast!(self, context,
                             FrameStreamId2, frame_stream_id2);
        }
    }

    #[inline]
    fn frame_stream_id2(&mut self, context: &mut ByteStream)
    -> Result<ParserValue, ParserError> {
        exit_if_eos!(self, context);

        self.bit_data1 <<= 8;
        self.bit_data1   = get_u8!(context) as u32;

        transition_fast!(self, context,
                         FrameStreamId3, frame_stream_id3);
    }

    #[inline]
    fn frame_stream_id3(&mut self, context: &mut ByteStream)
    -> Result<ParserValue, ParserError> {
        exit_if_eos!(self, context);

        self.bit_data1 <<= 8;
        self.bit_data1   = get_u8!(context) as u32;

        transition_fast!(self, context,
                         FrameStreamId4, frame_stream_id4);
    }

    #[inline]
    fn frame_stream_id4(&mut self, context: &mut ByteStream)
    -> Result<ParserValue, ParserError> {
        exit_if_eos!(self, context);

        self.bit_data1 <<= 8;
        self.bit_data1   = get_u8!(context) as u32;
        self.bit_data1   = u32::from_be(self.bit_data1);

        transition_fast!(self, context,
                         FrameEnd, frame_end);
    }

    #[inline]
    fn frame_end(&mut self, context: &mut ByteStream)
    -> Result<ParserValue, ParserError> {
        // match frame type
        match self.bit_data2 {
            FR_DATA => {
                if has_flag!(self, FL_PADDED) {
                    set_state!(self, DataPadLength, data_pad_length);
                } else {
                    set_state!(self, DataDataWithoutPadding, data_data_without_padding);
                }
            },
            FR_HEADERS => {
                if has_flag!(self, FL_PADDED) {
                    set_state!(self, HeadersPadLength, headers_pad_length);
                } else if has_flag!(self, FL_PRIORITY) {
                    set_state!(self, HeadersStreamId1, headers_stream_id1);
                } else {
                    set_state!(self,
                               HeadersFragmentWithoutPadding, headers_fragment_without_padding);
                }
            },
            FR_PRIORITY => {
                set_state!(self, PriorityStreamId1, priority_stream_id1);
            },
            FR_RST_STREAM => {
                set_state!(self, RstStreamErrorCode1, rst_stream_error_code1);
            },
            FR_SETTINGS => {
                set_state!(self, SettingsId1, settings_id1);
            },
            FR_PUSH_PROMISE => {
                if has_flag!(self, FL_PADDED) {
                    set_state!(self, PushPromisePadLength, push_promise_pad_length);
                } else {
                    set_state!(self, PushPromiseStreamId1, push_promise_stream_id1);
                }
            },
            FR_PING => {
                set_state!(self, PingData, ping_data);
            },
            FR_GO_AWAY => {
                set_state!(self, GoAwayStreamId1, go_away_stream_id1);
            },
            FR_WINDOW_UPDATE => {
                set_state!(self, WindowUpdateIncrement1, window_update_increment1);
            },
            FR_CONTINUATION => {
                set_state!(self, HeadersFragmentWithoutPadding, headers_fragment_without_padding);
            },
            _ => {
                if has_flag!(self, FL_PADDED) {
                    set_state!(self, UnknownPadLength, unknown_pad_length);
                } else {
                    set_state!(self, UnknownDataWithoutPadding, unknown_data_without_padding);
                }
            }
        }

        if self.handler.on_frame(self.length_flags & 0xFFFFFF,
                                 self.bit_data2,
                                 (self.length_flags >> 24) as u8,
                                 self.bit_data1 >> 31 == 1,
                                 self.bit_data1 & 0x7FFFFFFF) {
            // reset bit data
            self.bit_data1 = 0;
            self.bit_data2 = 0;

            transition_fast!(self, context);
        } else {
            // reset bit data
            self.bit_data1 = 0;
            self.bit_data2 = 0;

            exit_callback!(self, context);
        }
    }

    #[inline]
    fn frame_padding(&mut self, context: &mut ByteStream)
    -> Result<ParserValue, ParserError> {
        exit_if_eos!(self, context);

        if bs_available!(context) >= self.bit_data2 as usize {
            // consume remaining padding
            bs_jump!(context, self.bit_data2 as usize);

            transition!(self, context,
                        FrameLength1, frame_length1);
        } else {
            // consume remaining stream
            self.bit_data2 -= bs_available!(context) as u8;

            bs_jump!(context, bs_available!(context));

            exit_eos!(self, context);
        }
    }

    // ---------------------------------------------------------------------------------------------
    // DATA STATES
    // ---------------------------------------------------------------------------------------------

    #[inline]
    fn data_pad_length(&mut self, context: &mut ByteStream)
    -> Result<ParserValue, ParserError> {
        exit_if_eos!(self, context);

        self.bit_data2 = get_u8!(context);

        transition_fast!(self, context,
                         DataDataWithPadding, data_data_with_padding);
    }

    #[inline]
    fn data_data_without_padding(&mut self, context: &mut ByteStream)
    -> Result<ParserValue, ParserError> {
        exit_if_eos!(self, context);

        data_without_padding!(self, context, on_data_data);
    }

    #[inline]
    fn data_data_with_padding(&mut self, context: &mut ByteStream)
    -> Result<ParserValue, ParserError> {
        exit_if_eos!(self, context);

        data_with_padding!(self, context, on_data_data);
    }

    // ---------------------------------------------------------------------------------------------
    // GO AWAY STATES
    // ---------------------------------------------------------------------------------------------

    #[inline]
    fn go_away_stream_id1(&mut self, context: &mut ByteStream)
    -> Result<ParserValue, ParserError> {
        exit_eos!(self, context);
    }

    #[inline]
    fn go_away_stream_id2(&mut self, context: &mut ByteStream)
    -> Result<ParserValue, ParserError> {
        exit_eos!(self, context);
    }

    #[inline]
    fn go_away_stream_id3(&mut self, context: &mut ByteStream)
    -> Result<ParserValue, ParserError> {
        exit_eos!(self, context);
    }

    #[inline]
    fn go_away_stream_id4(&mut self, context: &mut ByteStream)
    -> Result<ParserValue, ParserError> {
        exit_eos!(self, context);
    }

    #[inline]
    fn go_away_error_code1(&mut self, context: &mut ByteStream)
    -> Result<ParserValue, ParserError> {
        exit_eos!(self, context);
    }

    #[inline]
    fn go_away_error_code2(&mut self, context: &mut ByteStream)
    -> Result<ParserValue, ParserError> {
        exit_eos!(self, context);
    }

    #[inline]
    fn go_away_error_code3(&mut self, context: &mut ByteStream)
    -> Result<ParserValue, ParserError> {
        exit_eos!(self, context);
    }

    #[inline]
    fn go_away_error_code4(&mut self, context: &mut ByteStream)
    -> Result<ParserValue, ParserError> {
        exit_eos!(self, context);
    }

    #[inline]
    fn go_away_debug_data(&mut self, context: &mut ByteStream)
    -> Result<ParserValue, ParserError> {
        exit_eos!(self, context);
    }

    // ---------------------------------------------------------------------------------------------
    // HEADERS STATES
    // ---------------------------------------------------------------------------------------------

    #[inline]
    fn headers_pad_length(&mut self, context: &mut ByteStream)
    -> Result<ParserValue, ParserError> {
        exit_eos!(self, context);
    }

    #[inline]
    fn headers_stream_id1(&mut self, context: &mut ByteStream)
    -> Result<ParserValue, ParserError> {
        exit_eos!(self, context);
    }

    #[inline]
    fn headers_stream_id2(&mut self, context: &mut ByteStream)
    -> Result<ParserValue, ParserError> {
        exit_eos!(self, context);
    }

    #[inline]
    fn headers_stream_id3(&mut self, context: &mut ByteStream)
    -> Result<ParserValue, ParserError> {
        exit_eos!(self, context);
    }

    #[inline]
    fn headers_stream_id4(&mut self, context: &mut ByteStream)
    -> Result<ParserValue, ParserError> {
        exit_eos!(self, context);
    }

    #[inline]
    fn headers_weight(&mut self, context: &mut ByteStream)
    -> Result<ParserValue, ParserError> {
        exit_eos!(self, context);
    }

    #[inline]
    fn headers_fragment_without_padding(&mut self, context: &mut ByteStream)
    -> Result<ParserValue, ParserError> {
        exit_eos!(self, context);
    }

    #[inline]
    fn headers_fragment_with_padding(&mut self, context: &mut ByteStream)
    -> Result<ParserValue, ParserError> {
        exit_eos!(self, context);
    }

    // ---------------------------------------------------------------------------------------------
    // PING STATES
    // ---------------------------------------------------------------------------------------------

    #[inline]
    fn ping_data(&mut self, context: &mut ByteStream)
    -> Result<ParserValue, ParserError> {
        exit_eos!(self, context);
    }

    // ---------------------------------------------------------------------------------------------
    // PRIORITY STATES
    // ---------------------------------------------------------------------------------------------

    #[inline]
    fn priority_stream_id1(&mut self, context: &mut ByteStream)
    -> Result<ParserValue, ParserError> {
        exit_eos!(self, context);
    }

    #[inline]
    fn priority_stream_id2(&mut self, context: &mut ByteStream)
    -> Result<ParserValue, ParserError> {
        exit_eos!(self, context);
    }

    #[inline]
    fn priority_stream_id3(&mut self, context: &mut ByteStream)
    -> Result<ParserValue, ParserError> {
        exit_eos!(self, context);
    }

    #[inline]
    fn priority_stream_id4(&mut self, context: &mut ByteStream)
    -> Result<ParserValue, ParserError> {
        exit_eos!(self, context);
    }

    #[inline]
    fn priority_weight(&mut self, context: &mut ByteStream)
    -> Result<ParserValue, ParserError> {
        exit_eos!(self, context);
    }

    // ---------------------------------------------------------------------------------------------
    // PUSH PROMISE STATES
    // ---------------------------------------------------------------------------------------------

    #[inline]
    fn push_promise_pad_length(&mut self, context: &mut ByteStream)
    -> Result<ParserValue, ParserError> {
        exit_eos!(self, context);
    }

    #[inline]
    fn push_promise_stream_id1(&mut self, context: &mut ByteStream)
    -> Result<ParserValue, ParserError> {
        exit_eos!(self, context);
    }

    #[inline]
    fn push_promise_stream_id2(&mut self, context: &mut ByteStream)
    -> Result<ParserValue, ParserError> {
        exit_eos!(self, context);
    }

    #[inline]
    fn push_promise_stream_id3(&mut self, context: &mut ByteStream)
    -> Result<ParserValue, ParserError> {
        exit_eos!(self, context);
    }

    #[inline]
    fn push_promise_stream_id4(&mut self, context: &mut ByteStream)
    -> Result<ParserValue, ParserError> {
        exit_eos!(self, context);
    }

    // ---------------------------------------------------------------------------------------------
    // RST STREAM STATES
    // ---------------------------------------------------------------------------------------------

    #[inline]
    fn rst_stream_error_code1(&mut self, context: &mut ByteStream)
    -> Result<ParserValue, ParserError> {
        exit_eos!(self, context);
    }

    #[inline]
    fn rst_stream_error_code2(&mut self, context: &mut ByteStream)
    -> Result<ParserValue, ParserError> {
        exit_eos!(self, context);
    }

    #[inline]
    fn rst_stream_error_code3(&mut self, context: &mut ByteStream)
    -> Result<ParserValue, ParserError> {
        exit_eos!(self, context);
    }

    #[inline]
    fn rst_stream_error_code4(&mut self, context: &mut ByteStream)
    -> Result<ParserValue, ParserError> {
        exit_eos!(self, context);
    }

    // ---------------------------------------------------------------------------------------------
    // SETTINGS STATES
    // ---------------------------------------------------------------------------------------------

    #[inline]
    fn settings_id1(&mut self, context: &mut ByteStream)
    -> Result<ParserValue, ParserError> {
        exit_eos!(self, context);
    }

    #[inline]
    fn settings_id2(&mut self, context: &mut ByteStream)
    -> Result<ParserValue, ParserError> {
        exit_eos!(self, context);
    }

    #[inline]
    fn settings_value1(&mut self, context: &mut ByteStream)
    -> Result<ParserValue, ParserError> {
        exit_eos!(self, context);
    }

    #[inline]
    fn settings_value2(&mut self, context: &mut ByteStream)
    -> Result<ParserValue, ParserError> {
        exit_eos!(self, context);
    }

    #[inline]
    fn settings_value3(&mut self, context: &mut ByteStream)
    -> Result<ParserValue, ParserError> {
        exit_eos!(self, context);
    }

    #[inline]
    fn settings_value4(&mut self, context: &mut ByteStream)
    -> Result<ParserValue, ParserError> {
        exit_eos!(self, context);
    }

    // ---------------------------------------------------------------------------------------------
    // UNKNOWN STATES
    // ---------------------------------------------------------------------------------------------

    #[inline]
    fn unknown_pad_length(&mut self, context: &mut ByteStream)
    -> Result<ParserValue, ParserError> {
        exit_eos!(self, context);
    }

    #[inline]
    fn unknown_data_without_padding(&mut self, context: &mut ByteStream)
    -> Result<ParserValue, ParserError> {
        exit_eos!(self, context);
    }

    #[inline]
    fn unknown_data_with_padding(&mut self, context: &mut ByteStream)
    -> Result<ParserValue, ParserError> {
        exit_eos!(self, context);
    }

    // ---------------------------------------------------------------------------------------------
    // WINDOW UPDATE STATES
    // ---------------------------------------------------------------------------------------------

    #[inline]
    fn window_update_increment1(&mut self, context: &mut ByteStream)
    -> Result<ParserValue, ParserError> {
        exit_eos!(self, context);
    }

    #[inline]
    fn window_update_increment2(&mut self, context: &mut ByteStream)
    -> Result<ParserValue, ParserError> {
        exit_eos!(self, context);
    }

    #[inline]
    fn window_update_increment3(&mut self, context: &mut ByteStream)
    -> Result<ParserValue, ParserError> {
        exit_eos!(self, context);
    }

    #[inline]
    fn window_update_increment4(&mut self, context: &mut ByteStream)
    -> Result<ParserValue, ParserError> {
        exit_eos!(self, context);
    }

    // ---------------------------------------------------------------------------------------------
    // DEAD STATE
    // ---------------------------------------------------------------------------------------------

    #[inline]
    fn dead(&mut self, _context: &mut ByteStream)
    -> Result<ParserValue, ParserError> {
        exit_error!(Dead);
    }
}
