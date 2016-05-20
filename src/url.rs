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

//! URL handling functions.

use byte::hex_to_byte;

use std::fmt;

// -------------------------------------------------------------------------------------------------
// STREAM MACROS
// -------------------------------------------------------------------------------------------------

// Collect all visible 7-bit bytes, which is any non-control byte with the exception of space.
macro_rules! collect_visible {
    ($stream:expr, $stream_index:expr, $stop1:expr, $stop2:expr, $stop3:expr, $stop4:expr,
     $byte_error:expr, $eof_block:block) => ({
        let mut byte;

        loop {
            if is_eof!($stream, $stream_index) {
                $eof_block
            }

            byte = next!($stream, $stream_index);

            if $stop1 == byte || $stop2 == byte || $stop3 == byte || $stop4 == byte {
                break;
            } else if is_non_visible!(byte) {
                exit_error!($byte_error(byte));
            }
        }

        byte
    });

    ($stream:expr, $stream_index:expr, $stop1:expr, $stop2:expr, $byte_error:expr,
     $eof_block:block) => ({
        let mut byte;

        loop {
            if is_eof!($stream, $stream_index) {
                $eof_block
            }

            byte = next!($stream, $stream_index);

            if $stop1 == byte || $stop2 == byte {
                break;
            } else if is_non_visible!(byte) {
                exit_error!($byte_error(byte));
            }
        }

        byte
    });
}

// Exit with an error.
macro_rules! exit_error {
    ($error:expr) => ({
        return Err($error);
    });
}

// Exit with OK status.
macro_rules! exit_ok {
    ($stream_index:expr) => ({
        return Ok($stream_index);
    });
}

// Indicates that a specified amount of bytes are available.
macro_rules! has_bytes {
    ($stream:expr, $stream_index:expr, $length:expr) => (
        $stream_index + $length <= $stream.len()
    );
}

// Indicates that we're at the end of the stream.
macro_rules! is_eof {
    ($stream:expr, $stream_index:expr) => (
        $stream_index == $stream.len()
    );
}

// Jump a specified amount of bytes.
macro_rules! jump_bytes {
    ($stream_index:expr, $length:expr) => ({
        $stream_index += $length;
    });
}

// Advance the stream one byte.
macro_rules! next {
    ($stream:expr, $stream_index:expr) => ({
        $stream_index += 1;

        $stream[$stream_index - 1]
    });
}

// -------------------------------------------------------------------------------------------------

/// Decoding errors.
pub enum DecodeError {
    /// Invalid byte.
    Byte(u8),

    /// Invalid hex sequence.
    HexSequence(u8)
}

impl fmt::Display for DecodeError {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            DecodeError::Byte(x) => {
                write!(formatter, "Invalid byte at {}", x)
            },
            DecodeError::HexSequence(x) => {
                write!(formatter, "Invalid hex sequence at {}", x)
            }
        }
    }
}

// -------------------------------------------------------------------------------------------------

/// Query errors.
pub enum QueryError {
    /// Invalid query field.
    Field(u8),

    /// Invalid query value.
    Value(u8)
}

impl fmt::Display for QueryError {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            QueryError::Field(x) => {
                write!(formatter, "Invalid query field at {}", x)
            },
            QueryError::Value(x) => {
                write!(formatter, "Invalid query value at {}", x)
            }
        }
    }
}

// -------------------------------------------------------------------------------------------------

/// Query segments.
pub enum QuerySegment<'a> {
    /// Flush segment.
    Flush,

    /// Field segment.
    Field(&'a [u8]),

    /// Value segment.
    Value(&'a [u8])
}

impl<'a> fmt::Display for QuerySegment<'a> {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            QuerySegment::Field(x) => {
                write!(formatter, "QuerySegment::Field({:?})", x)
            },
            QuerySegment::Flush => {
                write!(formatter, "QuerySegment::Flush")
            },
            QuerySegment::Value(x) => {
                write!(formatter, "QuerySegment::Flush({:?})", x)
            }
        }
    }
}

// -------------------------------------------------------------------------------------------------

/// URL errors.
pub enum UrlError {
    /// Invalid fragment.
    Fragment(u8),

    /// Invalid host.
    Host(u8),

    /// Invalid path.
    Path(u8),

    /// Invalid port.
    Port(u8),

    /// Invalid query string.
    QueryString(u8)
}

impl fmt::Display for UrlError {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            UrlError::Fragment(x) => {
                write!(formatter, "Invalid fragment at {}", x)
            },
            UrlError::Host(x) => {
                write!(formatter, "Invalid host at {}", x)
            },
            UrlError::Path(x) => {
                write!(formatter, "Invalid path at {}", x)
            },
            UrlError::Port(x) => {
                write!(formatter, "Invalid port at {}", x)
            },
            UrlError::QueryString(x) => {
                write!(formatter, "Invalid query string at {}", x)
            }
        }
    }
}

// -------------------------------------------------------------------------------------------------

/// URL segments.
pub enum UrlSegment<'a> {
    /// Fragment segment.
    Fragment(&'a [u8]),

    /// Host segment.
    Host(&'a [u8]),

    /// Path segment.
    Path(&'a [u8]),

    /// Port segment.
    Port(&'a [u8]),

    /// Query string segment.
    QueryString(&'a [u8])
}

impl<'a> fmt::Display for UrlSegment<'a> {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            UrlSegment::Fragment(x) => {
                write!(formatter, "UrlSegment::Fragment({:?})", x)
            },
            UrlSegment::Host(x) => {
                write!(formatter, "UrlSegment::Host({:?})", x)
            },
            UrlSegment::Path(x) => {
                write!(formatter, "UrlSegment::Path({:?})", x)
            },
            UrlSegment::Port(x) => {
                write!(formatter, "UrlSegment::Port({:?})", x)
            },
            UrlSegment::QueryString(x) => {
                write!(formatter, "UrlSegment::QueryString({:?})", x)
            }
        }
    }
}

// -------------------------------------------------------------------------------------------------

/// Decode a URL encoded stream of bytes.
pub fn decode<F>(stream: &[u8], mut append_fn: F) -> Result<usize, DecodeError>
where F : FnMut(&[u8]) {
    let mut byte;
    let mut mark_index;
    let mut stream_index = 0;

    loop {
        mark_index = stream_index;

        byte = collect_visible!(stream, stream_index,
                                b'%', b'+',
                                DecodeError::Byte,
                                {
            if mark_index < stream_index {
                append_fn(&stream[mark_index..stream_index]);
            }

            exit_ok!(stream_index);
        });

        if mark_index < stream_index - 1 {
            append_fn(&stream[mark_index..stream_index - 1]);
        }

        if byte == b'+' {
            append_fn(b" ");
        } else if has_bytes!(stream, stream_index, 2) {
            match hex_to_byte(&stream[stream_index..stream_index + 2]) {
                Some(byte) => {
                    jump_bytes!(stream_index, 2);

                    append_fn(&[byte]);
                },
                None => {
                    exit_error!(DecodeError::HexSequence(byte));
                }
            }
        } else {
            exit_error!(DecodeError::HexSequence(byte));
        }
    }
}

/// Parse a query string.
pub fn parse_query_string<F>(stream: &[u8], mut segment_fn: F) -> Result<usize, QueryError>
where F : FnMut(QuerySegment) {
    let mut byte;
    let mut mark_index;
    let mut stream_index = 0;

    loop {
        // field loop
        loop {
            mark_index = stream_index;

            byte = collect_visible!(stream, stream_index,
                                    b'%', b'+', b'=', b'&',
                                    QueryError::Field,
                                    {
                if mark_index < stream_index {
                    segment_fn(QuerySegment::Field(&stream[mark_index..stream_index]));
                }

                segment_fn(QuerySegment::Flush);

                exit_ok!(stream_index);
            });

            if mark_index < stream_index - 1 {
                segment_fn(QuerySegment::Field(&stream[mark_index..stream_index - 1]));
            }

            if byte == b'%' {
                if has_bytes!(stream, stream_index, 2) {
                    match hex_to_byte(&stream[stream_index..stream_index + 2]) {
                        Some(byte) => {
                            jump_bytes!(stream_index, 2);

                            segment_fn(QuerySegment::Field(&[byte]));
                        },
                        None => {
                            exit_error!(QueryError::Field(byte));
                        }
                    }
                } else {
                    exit_error!(QueryError::Field(byte));
                }
            } else if byte == b'+' {
                segment_fn(QuerySegment::Field(b" "));
            } else if byte == b'=' {
                if stream_index == 1 {
                    // first byte cannot be an equal sign
                    exit_error!(QueryError::Field(byte));
                }

                break;
            } else {
                if stream_index == 1 {
                    // first byte cannot be an ampersand
                    exit_error!(QueryError::Field(byte));
                }

                // field without a value, flush
                segment_fn(QuerySegment::Flush);
            }
        }

        // param loop
        loop {
            mark_index = stream_index;

            byte = collect_visible!(stream, stream_index,
                                    b'%', b'+', b'=', b'&',
                                    QueryError::Value,
                                    {
                if mark_index < stream_index {
                    segment_fn(QuerySegment::Value(&stream[mark_index..stream_index]));
                }

                segment_fn(QuerySegment::Flush);

                exit_ok!(stream_index);
            });

            if mark_index < stream_index - 1 {
                segment_fn(QuerySegment::Value(&stream[mark_index..stream_index - 1]));
            }

            if byte == b'%' {
                if has_bytes!(stream, stream_index, 2) {
                    match hex_to_byte(&stream[stream_index..stream_index + 2]) {
                        Some(byte) => {
                            jump_bytes!(stream_index, 2);

                            segment_fn(QuerySegment::Value(&[byte]));
                        },
                        None => {
                            exit_error!(QueryError::Value(byte));
                        }
                    }
                } else {
                    exit_error!(QueryError::Value(byte));
                }
            } else if byte == b'+' {
                segment_fn(QuerySegment::Value(b" "));
            } else if byte == b'=' {
                // value cannot have an equal sign
                exit_error!(QueryError::Value(byte));
            } else {
                break;
            }
        }
    }
}
