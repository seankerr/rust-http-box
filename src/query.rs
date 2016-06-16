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

//! Query handling functions.

use byte::hex_to_byte;
use byte_slice::ByteStream;

use std::{ fmt,
           str };

// -------------------------------------------------------------------------------------------------

/// Exit with Ok status.
macro_rules! exit_ok {
    ($context:expr) => ({
        return Ok($context.stream_index);
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

impl fmt::Debug for DecodeError {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            DecodeError::Byte(x) => {
                write!(formatter, "DecodeError::Byte(Invalid byte at {})", x)
            },
            DecodeError::HexSequence(x) => {
                write!(formatter, "DecodeError::HexSequence(Invalid hex sequence at {})", x)
            }
        }
    }
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

impl fmt::Debug for QueryError {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            QueryError::Field(x) => {
                write!(formatter, "QueryError::Field(Invalid query field at {})", x)
            },
            QueryError::Value(x) => {
                write!(formatter, "QueryError::Value(Invalid query value at {})", x)
            }
        }
    }
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
    /// Field segment.
    Field(&'a [u8]),

    /// Flush segment.
    Flush,

    /// Value segment.
    Value(&'a [u8])
}

impl<'a> fmt::Debug for QuerySegment<'a> {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            QuerySegment::Field(x) => {
                write!(formatter, "QuerySegment::Field({})", str::from_utf8(x).unwrap())
            },
            QuerySegment::Flush => {
                write!(formatter, "QuerySegment::Flush")
            },
            QuerySegment::Value(x) => {
                write!(formatter, "QuerySegment::Value({})", str::from_utf8(x).unwrap())
            }
        }
    }
}

impl<'a> fmt::Display for QuerySegment<'a> {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            QuerySegment::Field(x) => {
                write!(formatter, "{}", str::from_utf8(x).unwrap())
            },
            QuerySegment::Flush => {
                write!(formatter, "Flush")
            },
            QuerySegment::Value(x) => {
                write!(formatter, "{}", str::from_utf8(x).unwrap())
            }
        }
    }
}

// -------------------------------------------------------------------------------------------------

/// Decode a URL encoded stream of bytes.
pub fn decode<F>(bytes: &[u8], mut append_fn: F) -> Result<usize, DecodeError>
where F : FnMut(&[u8]) {
    let mut context = ByteStream::new(bytes);

    loop {
        bs_mark!(context);

        collect_visible!(context, DecodeError::Byte,
            // stop on these bytes
               context.byte == b'%'
            || context.byte == b'+',

            // on end-of-stream
            {
                if context.mark_index < context.stream_index {
                    append_fn(bs_slice!(context));
                }

                exit_ok!(context);
            }
        );

        if bs_slice_length!(context) > 1 {
            append_fn(bs_slice_ignore!(context));
        }

        if context.byte == b'+' {
            append_fn(b" ");
        } else if bs_has_bytes!(context, 2) {
            if let Some(byte) = hex_to_byte(bs_peek!(context, 2)) {
                bs_jump!(context, 2);

                append_fn(&[byte]);
            } else {
                return Err(DecodeError::HexSequence(context.byte));
            }
        } else {
            return Err(DecodeError::HexSequence(context.byte));
        }
    }
}

/// Parse a query.
pub fn parse_query<F>(query: &[u8], separator: u8, mut segment_fn: F) -> Result<usize, QueryError>
where F : FnMut(QuerySegment) {
    let mut context = ByteStream::new(query);

    loop {
        // field loop
        loop {
            bs_mark!(context);

            collect_visible!(context, QueryError::Field,
                // stop on these bytes
                   context.byte == b'%'
                || context.byte == b'+'
                || context.byte == b'='
                || context.byte == separator,

                // on end-of-stream
                {
                    if bs_slice_length!(context) > 0 {
                        segment_fn(QuerySegment::Field(bs_slice!(context)));
                    }

                    segment_fn(QuerySegment::Flush);

                    exit_ok!(context);
                }
            );

            if bs_slice_length!(context) > 1 {
                segment_fn(QuerySegment::Field(bs_slice_ignore!(context)));
            }

            if context.byte == b'%' {
                if bs_has_bytes!(context, 2) {
                    if let Some(byte) = hex_to_byte(bs_peek!(context, 2)) {
                        bs_jump!(context, 2);

                        segment_fn(QuerySegment::Field(&[byte]));
                    } else {
                        return Err(QueryError::Field(context.byte));
                    }
                } else {
                    return Err(QueryError::Field(context.byte));
                }
            } else if context.byte == b'+' {
                segment_fn(QuerySegment::Field(b" "));
            } else if context.byte == b'=' {
                if context.stream_index == 1 {
                    // first byte cannot be an equal sign
                    return Err(QueryError::Field(context.byte));
                }

                break;
            } else if context.stream_index == 1 {
                // first byte cannot be an ampersand
                return Err(QueryError::Field(context.byte));
            } else {
                // field without a value, flush
                segment_fn(QuerySegment::Flush);
            }
        }

        // param loop
        loop {
            bs_mark!(context);

            collect_visible!(context, QueryError::Value,
                // stop on these bytes
                   context.byte == b'%'
                || context.byte == b'+'
                || context.byte == b'='
                || context.byte == separator,

                // on end-of-stream
                {
                    if bs_slice_length!(context) > 0 {
                        segment_fn(QuerySegment::Value(bs_slice!(context)));
                    }

                    segment_fn(QuerySegment::Flush);

                    exit_ok!(context);
                }
            );

            if bs_slice_length!(context) > 1 {
                segment_fn(QuerySegment::Value(bs_slice_ignore!(context)));
            }

            if context.byte == b'%' {
                if bs_has_bytes!(context, 2) {
                    if let Some(byte) = hex_to_byte(bs_peek!(context, 2)) {
                        bs_jump!(context, 2);

                        segment_fn(QuerySegment::Value(&[byte]));
                    } else {
                        return Err(QueryError::Value(context.byte));
                    }
                } else {
                    return Err(QueryError::Value(context.byte));
                }
            } else if context.byte == b'+' {
                segment_fn(QuerySegment::Value(b" "));
            } else if context.byte == b'=' {
                // value cannot have an equal sign
                return Err(QueryError::Value(context.byte));
            } else {
                segment_fn(QuerySegment::Flush);

                break;
            }
        }
    }
}
