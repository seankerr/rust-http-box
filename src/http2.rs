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

//! HTTP 2.x parser, states, and errors.

#![allow(dead_code)]

use fsm::{ ParserValue,
           Success };

use byte_slice::ByteStream;
use std::{ fmt };

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

/// Unsupported error code.
const E_UNSUPPORTED: u8 = 0xFF;

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

/// Unsupported frame type.
const FR_UNSUPPORTED: u8 = 0xFF;

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

/// Unsupported setting.
const S_UNSUPPORTED: u16 = 0xFF;

// -------------------------------------------------------------------------------------------------
// MACROS
// -------------------------------------------------------------------------------------------------

/// Retrieve the remaining frame payload length sans padding data.
macro_rules! actual_length {
    ($parser:expr) => ({
        payload_length!($parser) - pad_length!($parser)
    });
}

/// Decrease payload length.
macro_rules! dec_payload_length {
    ($parser:expr, $length:expr) => ({
        $parser.bit_data32a = ($parser.bit_data32a & 0xFF)
                            | (($parser.bit_data32a >> 8) - $length) << 8;
    });
}

/// Retrieve a u8.
macro_rules! get_u8 {
    ($context:expr) => ({
        bs_jump!($context, 1);

        $context.stream[$context.stream_index - 1]
    });
}

/// Indicates that a flag is set.
macro_rules! has_flag {
    ($parser:expr, $flag:expr) => (
        $parser.bit_data8a & $flag == $flag
    );
}

/// Retrieve the pad length.
macro_rules! pad_length {
    ($parser:expr) => ({
        $parser.bit_data32a & 0xFF
    });
}

/// Retrieve the remaining frame payload length.
macro_rules! payload_length {
    ($parser:expr) => ({
        $parser.bit_data32a >> 8
    });
}

/// Parse payload data.
macro_rules! parse_payload_data {
    ($parser:expr, $context:expr, $callback:ident, $state:ident, $state_function:ident) => ({
        if bs_available!($context) >= actual_length!($parser) as usize {
            // collect remaining data
            bs_jump!($context, actual_length!($parser) as usize);

            dec_payload_length!($parser, actual_length!($parser));

            set_state!($parser, $state, $state_function);

            if $parser.handler.$callback(bs_slice!($context), true) {
                transition!($parser, $context);
            } else {
                exit_callback!($parser, $context);
            }
        }

        // collect remaining slice
        dec_payload_length!($parser, bs_available!($context) as u32);

        bs_jump!($context, bs_available!($context));

        if $parser.handler.$callback(bs_slice!($context), false) {
            exit_eos!($parser, $context);
        } else {
            exit_callback!($parser, $context);
        }
    });
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
#[derive(Clone,Copy,PartialEq)]
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
    StreamClosed = E_STREAM_CLOSED,

    /// Unsupported error.
    Unsupported = E_UNSUPPORTED
}

impl ErrorCode {
    /// Create a new `ErrorCode` from a `u8`.
    pub fn from_u8(byte: u8) -> ErrorCode {
        match byte {
            E_CANCEL              => ErrorCode::Cancel,
            E_COMPRESSION         => ErrorCode::Compression,
            E_CONNECT             => ErrorCode::Connect,
            E_ENHANCE_YOUR_CALM   => ErrorCode::EnhanceYourCalm,
            E_FLOW_CONTROL        => ErrorCode::FlowControl,
            E_FRAME_SIZE          => ErrorCode::FrameSize,
            E_HTTP_1_1_REQUIRED   => ErrorCode::Http11Required,
            E_INADEQUATE_SECURITY => ErrorCode::InadequateSecurity,
            E_INTERNAL            => ErrorCode::Internal,
            E_NO_ERROR            => ErrorCode::NoError,
            E_PROTOCOL            => ErrorCode::Protocol,
            E_REFUSED_STREAM      => ErrorCode::RefusedStream,
            E_SETTINGS_TIMEOUT    => ErrorCode::SettingsTimeout,
            E_STREAM_CLOSED       => ErrorCode::StreamClosed,
            _                     => ErrorCode::Unsupported
        }
    }

    /// Convert this error code to byte value.
    pub fn as_byte(&self) -> u8 {
        match *self {
            ErrorCode::Cancel             => E_CANCEL,
            ErrorCode::Compression        => E_COMPRESSION,
            ErrorCode::Connect            => E_CONNECT,
            ErrorCode::EnhanceYourCalm    => E_ENHANCE_YOUR_CALM,
            ErrorCode::FlowControl        => E_FLOW_CONTROL,
            ErrorCode::FrameSize          => E_FRAME_SIZE,
            ErrorCode::Http11Required     => E_HTTP_1_1_REQUIRED,
            ErrorCode::InadequateSecurity => E_INADEQUATE_SECURITY,
            ErrorCode::Internal           => E_INTERNAL,
            ErrorCode::NoError            => E_NO_ERROR,
            ErrorCode::Protocol           => E_PROTOCOL,
            ErrorCode::RefusedStream      => E_REFUSED_STREAM,
            ErrorCode::SettingsTimeout    => E_SETTINGS_TIMEOUT,
            ErrorCode::StreamClosed       => E_STREAM_CLOSED,
            ErrorCode::Unsupported        => E_UNSUPPORTED
        }
    }

    /// Indicates that this an `ErrorCode::Cancel`.
    pub fn is_cancel(&self) -> bool {
        *self == ErrorCode::Cancel
    }

    /// Indicates that this an `ErrorCode::Compression`.
    pub fn is_compression(&self) -> bool {
        *self == ErrorCode::Compression
    }

    /// Indicates that this an `ErrorCode::Connect`.
    pub fn is_connect(&self) -> bool {
        *self == ErrorCode::Connect
    }

    /// Indicates that this an `ErrorCode::EnhanceYourCalm`.
    pub fn is_enhance_your_calm(&self) -> bool {
        *self == ErrorCode::EnhanceYourCalm
    }

    /// Indicates that this an `ErrorCode::FlowControl`.
    pub fn is_flow_control(&self) -> bool {
        *self == ErrorCode::FlowControl
    }

    /// Indicates that this an `ErrorCode::FrameSize`.
    pub fn is_frame_size(&self) -> bool {
        *self == ErrorCode::FrameSize
    }

    /// Indicates that this an `ErrorCode::Http11Required`.
    pub fn is_http_1_1_required(&self) -> bool {
        *self == ErrorCode::Http11Required
    }

    /// Indicates that this an `ErrorCode::InadequateSecurity`.
    pub fn is_inadequate_security(&self) -> bool {
        *self == ErrorCode::InadequateSecurity
    }

    /// Indicates that this an `ErrorCode::Internal`.
    pub fn is_internal(&self) -> bool {
        *self == ErrorCode::Internal
    }

    /// Indicates that this an `ErrorCode::NoError`.
    pub fn is_no_error(&self) -> bool {
        *self == ErrorCode::NoError
    }

    /// Indicates that this an `ErrorCode::Protocol`.
    pub fn is_protocol(&self) -> bool {
        *self == ErrorCode::Protocol
    }

    /// Indicates that this an `ErrorCode::RefusedStream`.
    pub fn is_refused_stream(&self) -> bool {
        *self == ErrorCode::RefusedStream
    }

    /// Indicates that this an `ErrorCode::SettingsTimeout`.
    pub fn is_settings_timeout(&self) -> bool {
        *self == ErrorCode::SettingsTimeout
    }

    /// Indicates that this an `ErrorCode::StreamClosed`.
    pub fn is_stream_closed(&self) -> bool {
        *self == ErrorCode::StreamClosed
    }
}

// -------------------------------------------------------------------------------------------------

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
            "Flags(EndHeaders: {}, EndStream: {}, Padded: {}, Priority: {})",
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
            "Flags(EndHeaders: {}, EndStream: {}, Padded: {}, Priority: {})",
            self.flags & FL_END_HEADERS == FL_END_HEADERS,
            self.flags & FL_END_STREAM == FL_END_STREAM,
            self.flags & FL_PADDED == FL_PADDED,
            self.flags & FL_PRIORITY == FL_PRIORITY
        )
    }
}

// -------------------------------------------------------------------------------------------------

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

// -------------------------------------------------------------------------------------------------

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

// -------------------------------------------------------------------------------------------------

/// Type that handles HTTP/2.x parser events.
#[allow(unused_variables)]
pub trait HttpHandler {
    /// Callback that is executed when a data frame has been located.
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
    fn on_data(&mut self, data: &[u8], finished: bool) -> bool {
        true
    }

    /// Callback that is executed when a new frame has been located.
    ///
    /// **Arguments:**
    ///
    /// **`payload_length`**
    ///
    /// The payload length.
    ///
    /// **`frame_type`**
    ///
    /// The type.
    ///
    /// **`flags`**
    ///
    /// The flags.
    ///
    /// **`stream_id`**
    ///
    /// The stream identifier.
    ///
    /// **Returns:**
    ///
    /// `true` when parsing should continue, `false` to exit the parser function prematurely with
    /// [`Success::Callback`](../fsm/enum.Success.html#variant.Callback).
    fn on_frame(&mut self, payload_length: u32, frame_type: u8, flags: u8, stream_id: u32) -> bool {
        true
    }

    /// Callback that is executed when a go away frame has been located.
    ///
    /// **Arguments:**
    ///
    /// **`stream_id`**
    ///
    /// The stream identifier.
    ///
    /// **`error_code`**
    ///
    /// The error code.
    ///
    /// **Returns:**
    ///
    /// `true` when parsing should continue, `false` to exit the parser function prematurely with
    /// [`Success::Callback`](../fsm/enum.Success.html#variant.Callback).
    fn on_go_away(&mut self, stream_id: u32, error_code: u32) -> bool {
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

    /// Callback that is executed when a headers frame has been located.
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
    fn on_headers(&mut self, exclusive: bool, stream_id: u32) -> bool {
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

    /// Callback that is executed when a ping frame has been located.
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
    fn on_ping(&mut self, data: &[u8], finished: bool) -> bool {
        true
    }

    /// Callback that is executed when a priority frame has been located.
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
    /// **`weight`**
    ///
    /// The weight.
    ///
    /// **Returns:**
    ///
    /// `true` when parsing should continue, `false` to exit the parser function prematurely with
    /// [`Success::Callback`](../fsm/enum.Success.html#variant.Callback).
    fn on_priority(&mut self, exclusive: bool, stream_id: u32, weight: u8) -> bool {
        true
    }

    /// Callback that is executed when a push promise frame has been located.
    ///
    /// **Arguments:**
    ///
    /// **`stream_id`**
    ///
    /// The stream identifier.
    ///
    /// **Returns:**
    ///
    /// `true` when parsing should continue, `false` to exit the parser function prematurely with
    /// [`Success::Callback`](../fsm/enum.Success.html#variant.Callback).
    fn on_push_promise(&mut self, stream_id: u32) -> bool {
        true
    }

    /// Callback that is executed when a rst stream frame has been located.
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
    fn on_rst_stream(&mut self, error_code: u32) -> bool {
        true
    }

    /// Callback that is executed when a settings frame has been located.
    ///
    /// **Arguments:**
    ///
    /// **`id`**
    ///
    /// The identifier.
    ///
    /// **`value`**
    ///
    /// The value.
    ///
    /// **Returns:**
    ///
    /// `true` when parsing should continue, `false` to exit the parser function prematurely with
    /// [`Success::Callback`](../fsm/enum.Success.html#variant.Callback).
    fn on_settings(&mut self, id: u16, value: u32) -> bool {
        true
    }

    /// Callback that is executed when an unsupported frame has been located.
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
    fn on_unsupported(&mut self, data: &[u8], finished: bool) -> bool {
        true
    }

    /// Callback that is executed when a window update frame has been located.
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
    fn on_window_update(&mut self, size_increment: u32) -> bool {
        true
    }
}

// -------------------------------------------------------------------------------------------------

/// Parser error messages.
#[derive(Clone,Copy,PartialEq)]
pub enum ParserError {
    /// Parsing has failed.
    Dead
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
    // WINDOW UPDATE STATES
    // ---------------------------------------------------------------------------------------------

    /// Parsing window update increment first byte.
    WindowUpdateIncrement1,

    /// Parsing window update increment second byte.
    WindowUpdateIncrement2,

    /// Parsing window update increment third byte.
    WindowUpdateIncrement3,

    /// Parsing window update increment fourth byte.
    WindowUpdateIncrement4,

    // ---------------------------------------------------------------------------------------------
    // UNKNOWN STATES
    // ---------------------------------------------------------------------------------------------

    /// Parsing unsupported with padding.
    UnsupportedWithPadding,

    /// Parsing unsupported without padding.
    UnsupportedWithoutPadding,

    // ---------------------------------------------------------------------------------------------
    // FINISHED
    // ---------------------------------------------------------------------------------------------

    /// Parsing entire message has finished.
    Finished
}

// -------------------------------------------------------------------------------------------------

/// Setting.
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
            Setting::EnablePush(x) => {
                x
            },
            Setting::HeaderTableSize(x) => {
                x
            },
            Setting::InitialWindowSize(x) => {
                x
            },
            Setting::MaxConcurrentStreams(x) => {
                x
            },
            Setting::MaxFrameSize(x) => {
                x
            },
            Setting::MaxHeaderListSize(x) => {
                x
            },
            Setting::Unsupported(x) => {
                x
            }
        }
    }
}

impl fmt::Debug for Setting {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Setting::EnablePush(x) => {
                write!(formatter, "Setting::EnablePush({})", x)
            },
            Setting::HeaderTableSize(x) => {
                write!(formatter, "Setting::HeaderTableSize({})", x)
            },
            Setting::InitialWindowSize(x) => {
                write!(formatter, "Setting::InitialWindowSize({})", x)
            },
            Setting::MaxConcurrentStreams(x) => {
                write!(formatter, "Setting::MaxConcurrentStreams({})", x)
            },
            Setting::MaxFrameSize(x) => {
                write!(formatter, "Setting::MaxFrameSize({})", x)
            },
            Setting::MaxHeaderListSize(x) => {
                write!(formatter, "Setting::MaxHeaderListSize({})", x)
            },
            Setting::Unsupported(x) => {
                write!(formatter, "Setting::Unsupported({})", x)
            }
        }
    }
}

impl fmt::Display for Setting {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Setting::EnablePush(x) => {
                write!(formatter, "EnablePush({})", x)
            },
            Setting::HeaderTableSize(x) => {
                write!(formatter, "HeaderTableSize({})", x)
            },
            Setting::InitialWindowSize(x) => {
                write!(formatter, "InitialWindowSize({})", x)
            },
            Setting::MaxConcurrentStreams(x) => {
                write!(formatter, "MaxConcurrentStreams({})", x)
            },
            Setting::MaxFrameSize(x) => {
                write!(formatter, "MaxFrameSize({})", x)
            },
            Setting::MaxHeaderListSize(x) => {
                write!(formatter, "MaxHeaderListSize({})", x)
            },
            Setting::Unsupported(x) => {
                write!(formatter, "Unsupported({})", x)
            }
        }
    }
}

// -------------------------------------------------------------------------------------------------

/// HTTP 2.x parser.
pub struct Parser<'a, T: HttpHandler> {
    /// Bit data that stores parser state details.
    bit_data32a: u32,

    /// Bit data that stores parser state details.
    bit_data32b: u32,

    /// Bit data that stores parser state details.
    bit_data8a: u8,

    /// Bit data that stores parser state details.
    bit_data8b: u8,

    /// Total byte count processed.
    byte_count: usize,

    /// Handler implementation.
    handler: T,

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
    pub fn new(handler: T) -> Parser<'a, T> {
        Parser{ bit_data32a:    0,
                bit_data32b:    0,
                bit_data8a:     0,
                bit_data8b:     0,
                byte_count:     0,
                handler:        handler,
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
        self.byte_count     = 0;
        self.state          = ParserState::FrameLength1;
        self.state_function = Parser::frame_length1;

        self.reset_bit_data();
    }

    /// Reset bit data.
    fn reset_bit_data(&mut self) {
        self.bit_data32a = 0;
        self.bit_data32b = 0;
        self.bit_data8a  = 0;
        self.bit_data8b  = 0;
    }

    /// Resume parsing an additional slice of data.
    ///
    /// # Arguments
    ///
    /// **`stream`**
    ///
    /// The stream of data to be parsed.
    #[inline]
    pub fn resume(&mut self, stream: &[u8]) -> Result<Success, ParserError> {
        let mut context = ByteStream::new(stream);

        loop {
            match (self.state_function)(self, &mut context) {
                Ok(ParserValue::Continue) => {
                },
                Ok(ParserValue::Exit(success)) => {
                    self.byte_count += context.stream_index;

                    if let Success::Finished(_) = success {
                        self.state = ParserState::Finished;
                    }

                    return Ok(success);
                },
                Err(error) => {
                    self.byte_count     += context.stream_index;
                    self.state           = ParserState::Dead;
                    self.state_function  = Parser::dead;

                    return Err(error);
                }
            }
        }
    }

    /// Retrieve the current state.
    pub fn state(&self) -> ParserState {
        self.state
    }

    // ---------------------------------------------------------------------------------------------
    // FRAME STATES
    // ---------------------------------------------------------------------------------------------

    #[inline]
    fn frame_length1(&mut self, context: &mut ByteStream)
    -> Result<ParserValue, ParserError> {
        exit_if_eos!(self, context);

        self.reset_bit_data();

        if bs_available!(context) >= 4 {
            // read entire length and type
            read_u32!(context, self.bit_data32a);

            transition_fast!(
                self,
                context,
                FrameFlags,
                frame_flags
            )
        }

        // read first length byte
        self.bit_data32a |= (get_u8!(context) as u32) << 24;

        transition_fast!(
            self,
            context,
            FrameLength2,
            frame_length2
        );
    }

    #[inline]
    fn frame_length2(&mut self, context: &mut ByteStream)
    -> Result<ParserValue, ParserError> {
        exit_if_eos!(self, context);

        self.bit_data32a |= (get_u8!(context) as u32) << 16;

        transition_fast!(
            self,
            context,
            FrameLength3,
            frame_length3
        );
    }

    #[inline]
    fn frame_length3(&mut self, context: &mut ByteStream)
    -> Result<ParserValue, ParserError> {
        exit_if_eos!(self, context);

        self.bit_data32a |= (get_u8!(context) as u32) << 8;

        transition_fast!(
            self,
            context,
            FrameType,
            frame_type
        );
    }

    #[inline]
    fn frame_type(&mut self, context: &mut ByteStream)
    -> Result<ParserValue, ParserError> {
        exit_if_eos!(self, context);

        self.bit_data32a |= get_u8!(context) as u32;

        transition_fast!(
            self,
            context,
            FrameFlags,
            frame_flags
        );
    }

    #[inline]
    fn frame_flags(&mut self, context: &mut ByteStream)
    -> Result<ParserValue, ParserError> {
        exit_if_eos!(self, context);

        self.bit_data8a = get_u8!(context);

        transition_fast!(
            self,
            context,
            FrameStreamId1,
            frame_stream_id1
        );
    }

    #[inline]
    fn frame_stream_id1(&mut self, context: &mut ByteStream)
    -> Result<ParserValue, ParserError> {
        exit_if_eos!(self, context);

        if bs_available!(context) >= 4 {
            // read entire stream id
            read_u32!(context, self.bit_data32b);

            transition_fast!(
                self,
                context,
                FrameEnd,
                frame_end
            );
        }

        // read first stream id byte
        self.bit_data32b = (get_u8!(context) as u32) << 24;

        transition_fast!(
            self,
            context,
            FrameStreamId2,
            frame_stream_id2
        );
    }

    #[inline]
    fn frame_stream_id2(&mut self, context: &mut ByteStream)
    -> Result<ParserValue, ParserError> {
        exit_if_eos!(self, context);

        self.bit_data32b = (get_u8!(context) as u32) << 16;

        transition_fast!(
            self,
            context,
            FrameStreamId3,
            frame_stream_id3
        );
    }

    #[inline]
    fn frame_stream_id3(&mut self, context: &mut ByteStream)
    -> Result<ParserValue, ParserError> {
        exit_if_eos!(self, context);

        self.bit_data32b = (get_u8!(context) as u32) << 8;

        transition_fast!(
            self,
            context,
            FrameStreamId4,
            frame_stream_id4
        );
    }

    #[inline]
    fn frame_stream_id4(&mut self, context: &mut ByteStream)
    -> Result<ParserValue, ParserError> {
        exit_if_eos!(self, context);

        self.bit_data32b |= get_u8!(context) as u32;

        transition_fast!(
            self,
            context,
            FrameEnd,
            frame_end
        );
    }

    #[inline]
    fn frame_end(&mut self, context: &mut ByteStream)
    -> Result<ParserValue, ParserError> {
        // match frame type
        match (self.bit_data32a & 0xFF) as u8 {
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
                    set_state!(self, UnsupportedWithPadding, unsupported_with_padding);
                } else {
                    set_state!(self, UnsupportedWithoutPadding, unsupported_without_padding);
                }
            }
        }

        if self.handler.on_frame(payload_length!(self),
                                 (self.bit_data32a & 0xFF) as u8,
                                 self.bit_data8a,
                                 self.bit_data32b & 0x7FFFFFFF) {
            transition_fast!(self, context);
        } else {
            exit_callback!(self, context);
        }
    }

    #[inline]
    fn frame_padding(&mut self, context: &mut ByteStream)
    -> Result<ParserValue, ParserError> {
        exit_if_eos!(self, context);

        if bs_available!(context) >= self.bit_data8a as usize {
            // consume remaining padding
            bs_jump!(context, self.bit_data8a as usize);

            transition!(
                self,
                context,
                FrameLength1,
                frame_length1
            );
        } else {
            // consume remaining stream
            self.bit_data8a -= bs_available!(context) as u8;

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

        self.bit_data32a |= get_u8!(context) as u32;

        dec_payload_length!(self, 1);

        transition_fast!(
            self,
            context,
            DataDataWithPadding,
            data_data_with_padding
        );
    }

    #[inline]
    fn data_data_with_padding(&mut self, context: &mut ByteStream)
    -> Result<ParserValue, ParserError> {
        exit_if_eos!(self, context);

        parse_payload_data!(
            self,
            context,
            on_data,
            FramePadding,
            frame_padding
        );
    }

    #[inline]
    fn data_data_without_padding(&mut self, context: &mut ByteStream)
    -> Result<ParserValue, ParserError> {
        exit_if_eos!(self, context);

        parse_payload_data!(
            self,
            context,
            on_data,
            FrameLength1,
            frame_length1
        );
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
    // UNSUPPORTED STATES
    // ---------------------------------------------------------------------------------------------

    #[inline]
    fn unsupported_with_padding(&mut self, context: &mut ByteStream)
    -> Result<ParserValue, ParserError> {
        exit_if_eos!(self, context);

        parse_payload_data!(
            self,
            context,
            on_unsupported,
            FramePadding,
            frame_padding
        );
    }

    #[inline]
    fn unsupported_without_padding(&mut self, context: &mut ByteStream)
    -> Result<ParserValue, ParserError> {
        exit_if_eos!(self, context);

        parse_payload_data!(
            self,
            context,
            on_unsupported,
            FrameLength1,
            frame_length1
        );
    }

    // ---------------------------------------------------------------------------------------------
    // DEAD AND FINISHED STATE
    // ---------------------------------------------------------------------------------------------

    #[inline]
    fn dead(&mut self, _context: &mut ByteStream)
    -> Result<ParserValue, ParserError> {
        exit_error!(Dead);
    }

    #[inline]
    fn finished(&mut self, context: &mut ByteStream)
    -> Result<ParserValue, ParserError> {
        exit_finished!(self, context);
    }
}
