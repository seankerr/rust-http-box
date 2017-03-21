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

/// Cancel error code.
pub const E_CANCEL: u8 = 0x8;

/// Compression error code.
pub const E_COMPRESSION: u8 = 0x9;

/// Connect error code.
pub const E_CONNECT: u8 = 0xA;

/// Enhance your calm error code.
pub const E_ENHANCE_YOUR_CALM: u8 = 0xB;

/// Flow control error code.
pub const E_FLOW_CONTROL: u8 = 0x3;

/// Frame size error code.
pub const E_FRAME_SIZE: u8 = 0x6;

/// HTTP/1.1 required error code.
pub const E_HTTP_1_1_REQUIRED: u8 = 0xD;

/// Inadequate security error code.
pub const E_INADEQUATE_SECURITY: u8 = 0xC;

/// No error code.
pub const E_NO_ERROR: u8 = 0x0;

/// Internal error code.
pub const E_INTERNAL: u8 = 0x2;

/// Protocol error code.
pub const E_PROTOCOL: u8 = 0x1;

/// Refused stream error code.
pub const E_REFUSED_STREAM: u8 = 0x7;

/// Settings timeout error code.
pub const E_SETTINGS_TIMEOUT: u8 = 0x4;

/// Stream closed error code.
pub const E_STREAM_CLOSED: u8 = 0x5;

/// Unsupported error code.
pub const E_UNSUPPORTED: u8 = 0xFF;

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

impl fmt::Debug for ErrorCode {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            ErrorCode::Cancel => {
                write!(
                    formatter,
                    "<ErrorCode::Cancel>"
                )
            },
            ErrorCode::Compression => {
                write!(
                    formatter,
                    "<ErrorCode::Compression>"
                )
            },
            ErrorCode::Connect => {
                write!(
                    formatter,
                    "<ErrorCode::Connect>"
                )
            },
            ErrorCode::EnhanceYourCalm => {
                write!(
                    formatter,
                    "<ErrorCode::EnhanceYourCalm>"
                )
            },
            ErrorCode::FlowControl => {
                write!(
                    formatter,
                    "<ErrorCode::FlowControl>"
                )
            },
            ErrorCode::FrameSize => {
                write!(
                    formatter,
                    "<ErrorCode::FrameSize>"
                )
            },
            ErrorCode::Http11Required => {
                write!(
                    formatter,
                    "<ErrorCode::Http11Required>"
                )
            },
            ErrorCode::InadequateSecurity => {
                write!(
                    formatter,
                    "<ErrorCode::InadequateSecurity>"
                )
            },
            ErrorCode::Internal => {
                write!(
                    formatter,
                    "<ErrorCode::Internal>"
                )
            },
            ErrorCode::NoError => {
                write!(
                    formatter,
                    "<ErrorCode::NoError>"
                )
            },
            ErrorCode::Protocol => {
                write!(
                    formatter,
                    "<ErrorCode::Protocol>"
                )
            },
            ErrorCode::RefusedStream => {
                write!(
                    formatter,
                    "<ErrorCode::RefusedStream>"
                )
            },
            ErrorCode::SettingsTimeout => {
                write!(
                    formatter,
                    "<ErrorCode::SettingsTimeout>"
                )
            },
            ErrorCode::StreamClosed => {
                write!(
                    formatter,
                    "<ErrorCode::StreamClosed>"
                )
            },
            ErrorCode::Unsupported => {
                write!(
                    formatter,
                    "<ErrorCode::Unsupported>"
                )
            }
        }
    }
}

impl fmt::Display for ErrorCode {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            ErrorCode::Cancel => {
                write!(
                    formatter,
                    "<Cancel>"
                )
            },
            ErrorCode::Compression => {
                write!(
                    formatter,
                    "<Compression>"
                )
            },
            ErrorCode::Connect => {
                write!(
                    formatter,
                    "<Connect>"
                )
            },
            ErrorCode::EnhanceYourCalm => {
                write!(
                    formatter,
                    "<EnhanceYourCalm>"
                )
            },
            ErrorCode::FlowControl => {
                write!(
                    formatter,
                    "<FlowControl>"
                )
            },
            ErrorCode::FrameSize => {
                write!(
                    formatter,
                    "<FrameSize>"
                )
            },
            ErrorCode::Http11Required => {
                write!(
                    formatter,
                    "<Http11Required>"
                )
            },
            ErrorCode::InadequateSecurity => {
                write!(
                    formatter,
                    "<InadequateSecurity>"
                )
            },
            ErrorCode::Internal => {
                write!(
                    formatter,
                    "<Internal>"
                )
            },
            ErrorCode::NoError => {
                write!(
                    formatter,
                    "<NoError>"
                )
            },
            ErrorCode::Protocol => {
                write!(
                    formatter,
                    "<Protocol>"
                )
            },
            ErrorCode::RefusedStream => {
                write!(
                    formatter,
                    "<RefusedStream>"
                )
            },
            ErrorCode::SettingsTimeout => {
                write!(
                    formatter,
                    "<SettingsTimeout>"
                )
            },
            ErrorCode::StreamClosed => {
                write!(
                    formatter,
                    "<StreamClosed>"
                )
            },
            ErrorCode::Unsupported => {
                write!(
                    formatter,
                    "<Unsupported>"
                )
            }
        }
    }
}
