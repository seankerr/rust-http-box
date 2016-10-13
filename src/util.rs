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

//! Utility functions.
//!
//! This module provides support for decoding URL encoded data, parsing header fields, and parsing
//! query strings.

use byte::is_token;
use byte_slice::ByteStream;

use std::{ fmt,
           str };

// -------------------------------------------------------------------------------------------------

/// If the stream is EOS, exit with Ok status. Otherwise do nothing.
macro_rules! exit_if_eos {
    ($context:expr) => ({
        if bs_is_eos!($context) {
            exit_ok!($context);
        }
    });
}

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

/// Field errors.
pub enum FieldError {
    /// Invalid field name.
    Name(u8),

    /// Invalid field value.
    Value(u8)
}

impl fmt::Debug for FieldError {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            FieldError::Name(x) => {
                write!(formatter, "FieldError::Name(Invalid field name at {})", x)
            },
            FieldError::Value(x) => {
                write!(formatter, "FieldError::Value(Invalid field value at {})", x)
            }
        }
    }
}

impl fmt::Display for FieldError {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            FieldError::Name(x) => {
                write!(formatter, "Invalid field name at {}", x)
            },
            FieldError::Value(x) => {
                write!(formatter, "Invalid field value at {}", x)
            }
        }
    }
}

// -------------------------------------------------------------------------------------------------

/// Field segments.
pub enum FieldSegment<'a> {
    /// Name without a value.
    Name(&'a [u8]),

    /// Name and value pair.
    NameValue(&'a [u8], &'a [u8])
}

impl<'a> FieldSegment<'a> {
    /// Indicates that this [`FieldSegment`] contains a value.
    pub fn has_value(&self) -> bool {
        match *self {
            FieldSegment::Name(_) => false,
            _ => true
        }
    }

    /// Retrieve the name.
    pub fn name(&self) -> &'a [u8] {
        match *self {
            FieldSegment::Name(ref name) => name,
            FieldSegment::NameValue(ref name, _) => name
        }
    }

    /// Retrieve the value.
    pub fn value(&self) -> Option<&'a [u8]> {
        match *self {
            FieldSegment::NameValue(_, ref value) => Some(value),
            _ => None
        }
    }
}

impl<'a> fmt::Debug for FieldSegment<'a> {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            FieldSegment::Name(x) => {
                write!(formatter, "FieldSegment::Name({:?})", str::from_utf8(x).unwrap())
            },
            FieldSegment::NameValue(x,y) => {
                write!(formatter, "FieldSegment::NameValue({:?}, {:?})",
                       str::from_utf8(x).unwrap(),
                       str::from_utf8(y).unwrap())
            }
        }
    }
}

impl<'a> fmt::Display for FieldSegment<'a> {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            FieldSegment::Name(x) => {
                write!(formatter, "{:?}", str::from_utf8(x).unwrap())
            },
            FieldSegment::NameValue(x,y) => {
                write!(formatter, "{:?} = {:?}",
                       str::from_utf8(x).unwrap(),
                       str::from_utf8(y).unwrap())
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
    /// Field without a value.
    Field(&'a [u8]),

    /// Field and value pair.
    FieldValue(&'a [u8], &'a [u8])
}

impl<'a> QuerySegment<'a> {
    /// Retrieve the field.
    pub fn field(&self) -> &'a [u8] {
        match *self {
            QuerySegment::Field(ref field) => field,
            QuerySegment::FieldValue(ref field, _) => field
        }
    }

    /// Indicates that this [`QuerySegment`] contains a value.
    pub fn has_value(&self) -> bool {
        match *self {
            QuerySegment::Field(_) => false,
            _ => true
        }
    }

    /// Retrieve the value.
    pub fn value(&self) -> Option<&'a [u8]> {
        match *self {
            QuerySegment::FieldValue(_, ref value) => Some(value),
            _ => None
        }
    }
}

impl<'a> fmt::Debug for QuerySegment<'a> {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            QuerySegment::Field(x) => {
                write!(formatter, "QuerySegment::Field({:?})", str::from_utf8(x).unwrap())
            },
            QuerySegment::FieldValue(x,y) => {
                write!(formatter, "QuerySegment::FieldValue({:?}, {:?})",
                       str::from_utf8(x).unwrap(),
                       str::from_utf8(y).unwrap())
            }
        }
    }
}

impl<'a> fmt::Display for QuerySegment<'a> {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            QuerySegment::Field(x) => {
                write!(formatter, "{:?}", str::from_utf8(x).unwrap())
            },
            QuerySegment::FieldValue(x,y) => {
                write!(formatter, "{:?} = {:?}",
                       str::from_utf8(x).unwrap(),
                       str::from_utf8(y).unwrap())
            }
        }
    }
}

// -------------------------------------------------------------------------------------------------

/// Decode a URL encoded slice of bytes.
///
/// *Note:* `slice_fn` may be called multiple times in order to supply the entire piece of decoded
///         data.
///
/// # Arguments
///
/// **`bytes`**
///
/// The data to decode.
///
/// **`slice_fn`**
///
/// A closure that receives slices of decoded data.
///
/// # Returns
///
/// **`usize`**
///
/// The amount of data that was parsed.
///
/// # Errors
///
/// - [`DecodeError::Byte`](enum.DecodeError.html#variant.Byte)
/// - [`DecodeError::HexSequence`](enum.DecodeError.html#variant.HexSequence)
///
/// # Examples
///
/// ```
/// use http_box::util;
///
/// util::decode(b"fancy%20url%20encoded%20data%2E",
///     |s| {
///         // `s` is the most current slice of decoded data
///     }
/// );
/// ```
pub fn decode<F>(bytes: &[u8], mut slice_fn: F) -> Result<usize, DecodeError>
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
                    slice_fn(bs_slice!(context));
                }

                exit_ok!(context);
            }
        );

        if bs_slice_length!(context) > 1 {
            slice_fn(bs_slice_ignore!(context));
        }

        if context.byte == b'+' {
            slice_fn(b" ");
        } else if bs_has_bytes!(context, 2) {
            bs_next!(context);

            let mut byte = if is_digit!(context.byte) {
                (context.byte - b'0') << 4
            } else if b'@' < context.byte && context.byte < b'G' {
                (context.byte - 0x37) << 4
            } else if b'`' < context.byte && context.byte < b'g' {
                (context.byte - 0x57) << 4
            } else {
                return Err(DecodeError::HexSequence(context.byte));
            } as u8;

            bs_next!(context);

            byte |= if is_digit!(context.byte) {
                context.byte - b'0'
            } else if b'@' < context.byte && context.byte < b'G' {
                context.byte - 0x37
            } else if b'`' < context.byte && context.byte < b'g' {
                context.byte - 0x57
            } else {
                return Err(DecodeError::HexSequence(context.byte));
            } as u8;

            slice_fn(&[byte]);
        } else {
            if bs_has_bytes!(context, 1) {
                bs_next!(context);
            }

            return Err(DecodeError::HexSequence(context.byte));
        }
    }
}

/// Parse the content of a header field.
///
/// *Note:* This will normalize all names so that upper-cased bytes are converted to lower-case.
///
/// # Arguments
///
/// **`field`**
///
/// The field data to be parsed.
///
/// **`delimiter`**
///
/// The delimiting byte.
///
/// **`segment_fn`**
///
/// A closure that receives instances of [`FieldSegment`](enum.FieldSegment.html).
///
/// # Returns
///
/// **`usize`**
///
/// The amount of field data that was parsed.
///
/// # Errors
///
/// - [`FieldError::Name`](enum.FieldError.html#variant.Name)
/// - [`FieldError::Value`](enum.FieldError.html#variant.Value)
///
/// # Examples
///
/// ```
/// use http_box::util::FieldSegment;
/// use http_box::util;
///
/// util::parse_field(b"name-no-value; name1=value1; name2=\"value2\"", b';',
///     |s| {
///         if s.has_value() {
///             s.name();
///             s.value().unwrap();
///         } else {
///             s.name();
///         }
///     }
/// );
/// ```
pub fn parse_field<F>(field: &[u8], delimiter: u8, mut segment_fn: F) -> Result<usize, FieldError>
where F : FnMut(FieldSegment) {
    let mut context = ByteStream::new(field);
    let mut name    = Vec::new();
    let mut value   = Vec::new();

    loop {
        // parsing name
        consume_spaces!(context,
            // on end-of-stream
            exit_ok!(context)
        );

        bs_mark!(context);

        collect_tokens!(context, FieldError::Name,
            // stop on these bytes
               context.byte == b'='
            || context.byte == delimiter
            || (context.byte > 0x40 && context.byte < 0x5B),

            // on end-of-stream
            {
                // name without a value
                if bs_slice_length!(context) > 0 {
                    name.extend_from_slice(bs_slice!(context));
                }

                segment_fn(FieldSegment::Name(&name));

                exit_ok!(context);
            }
        );

        name.extend_from_slice(bs_slice_ignore!(context));

        if context.byte == b'=' {
            // parsing value
            exit_if_eos!(context);
            bs_next!(context);

            if context.byte == b'"' {
                // quoted value
                loop {
                    bs_mark!(context);

                    collect_quoted_field!(context, FieldError::Value,
                        // on end-of-stream
                        // didn't find an ending quote
                        return Err(FieldError::Value(context.byte))
                    );

                    if context.byte == b'"' {
                        // found end quote
                        value.extend_from_slice(bs_slice_ignore!(context));

                        segment_fn(FieldSegment::NameValue(&name, &value));

                        name.clear();
                        value.clear();

                        consume_spaces!(context,
                            // on end-of-stream
                            exit_ok!(context)
                        );

                        exit_if_eos!(context);
                        bs_next!(context);

                        if context.byte == delimiter {
                            break;
                        }

                        // expected a semicolon to end the value
                        return Err(FieldError::Value(context.byte));
                    } else {
                        // found backslash
                        if bs_is_eos!(context) {
                            return Err(FieldError::Value(context.byte));
                        }

                        value.extend_from_slice(bs_slice_ignore!(context));

                        bs_next!(context);

                        value.push(context.byte);
                    }
                }
            } else {
                // unquoted value
                bs_replay!(context);

                consume_spaces!(context,
                    // on end-of-stream
                    exit_ok!(context)
                );

                bs_mark!(context);

                collect_field!(context, FieldError::Value, delimiter,
                    // on end-of-stream
                    {
                        if bs_slice_length!(context) > 0 {
                            value.extend_from_slice(bs_slice!(context));
                        }

                        segment_fn(FieldSegment::NameValue(&name, &value));

                        exit_ok!(context);
                    }
                );

                if bs_slice_length!(context) == 0 {
                    // name without a value
                    segment_fn(FieldSegment::Name(&name));
                } else {
                    // name/value pair
                    segment_fn(FieldSegment::NameValue(&name, bs_slice_ignore!(context)));
                }

                name.clear();
                value.clear();
            }
        } else if context.byte == delimiter {
            // name without a value
            segment_fn(FieldSegment::Name(&name));

            name.clear();
        } else {
            // upper-cased byte, let's lower-case it
            name.push(context.byte + 0x20);
        }
    }
}

/// Parse a query.
///
/// # Arguments
///
/// **`query`**
///
/// The query data to be parsed.
///
/// **`delimiter`**
///
/// The delimiting byte.
///
/// **`segment_fn`**
///
/// A closure that receives instances of [`QuerySegment`](enum.QuerySegment.html).
///
/// # Returns
///
/// **`usize`**
///
/// The amount of query data that was parsed.
///
/// # Errors
///
/// - [`QueryError::Field`](enum.QueryError.html#variant.Field)
/// - [`QueryError::Value`](enum.QueryError.html#variant.Value)
///
/// # Examples
///
/// ```
/// use http_box::util::QuerySegment;
/// use http_box::util;
///
/// util::parse_query(b"field1-no-value&field2=value2&field%203=value%203", b'&',
///     |s| {
///         if s.has_value() {
///             s.field();
///             s.value().unwrap();
///         } else {
///             s.field();
///         }
///     }
/// );
/// ```
pub fn parse_query<F>(query: &[u8], delimiter: u8, mut segment_fn: F) -> Result<usize, QueryError>
where F : FnMut(QuerySegment) {
    let mut context = ByteStream::new(query);
    let mut name    = Vec::new();
    let mut value   = Vec::new();

    loop {
        // field loop
        loop {
            bs_mark!(context);

            collect_visible!(context, QueryError::Field,
                // stop on these bytes
                   context.byte == b'%'
                || context.byte == b'+'
                || context.byte == b'='
                || context.byte == delimiter,

                // on end-of-stream
                {
                    if bs_slice_length!(context) > 0 {
                        name.extend_from_slice(bs_slice!(context));
                    }

                    segment_fn(QuerySegment::Field(&name));

                    exit_ok!(context);
                }
            );

            if bs_slice_length!(context) > 1 {
                name.extend_from_slice(bs_slice_ignore!(context));
            }

            if context.byte == b'%' {
                if bs_has_bytes!(context, 2) {
                    bs_next!(context);

                    let mut byte = if is_digit!(context.byte) {
                        (context.byte - b'0') << 4
                    } else if b'@' < context.byte && context.byte < b'G' {
                        (context.byte - 0x37) << 4
                    } else if b'`' < context.byte && context.byte < b'g' {
                        (context.byte - 0x57) << 4
                    } else {
                        return Err(QueryError::Field(context.byte));
                    } as u8;

                    bs_next!(context);

                    byte |= if is_digit!(context.byte) {
                        context.byte - b'0'
                    } else if b'@' < context.byte && context.byte < b'G' {
                        context.byte - 0x37
                    } else if b'`' < context.byte && context.byte < b'g' {
                        context.byte - 0x57
                    } else {
                        return Err(QueryError::Field(context.byte));
                    } as u8;

                    name.push(byte);
                } else {
                    if bs_has_bytes!(context, 1) {
                        bs_next!(context);
                    }

                    return Err(QueryError::Field(context.byte));
                }
            } else if context.byte == b'+' {
                name.push(b' ');
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
                // field without a value
                segment_fn(QuerySegment::Field(&name));

                name.clear();
            }
        }

        // value loop
        loop {
            bs_mark!(context);

            collect_visible!(context, QueryError::Value,
                // stop on these bytes
                   context.byte == b'%'
                || context.byte == b'+'
                || context.byte == delimiter,

                // on end-of-stream
                {
                    if bs_slice_length!(context) > 0 {
                        value.extend_from_slice(bs_slice!(context));
                    }

                    segment_fn(QuerySegment::FieldValue(&name, &value));

                    exit_ok!(context);
                }
            );

            if bs_slice_length!(context) > 1 {
                value.extend_from_slice(bs_slice_ignore!(context));
            }

            if context.byte == b'%' {
                if bs_has_bytes!(context, 2) {
                    bs_next!(context);

                    let mut byte = if is_digit!(context.byte) {
                        (context.byte - b'0') << 4
                    } else if b'@' < context.byte && context.byte < b'G' {
                        (context.byte - 0x37) << 4
                    } else if b'`' < context.byte && context.byte < b'g' {
                        (context.byte - 0x57) << 4
                    } else {
                        return Err(QueryError::Value(context.byte));
                    } as u8;

                    bs_next!(context);

                    byte |= if is_digit!(context.byte) {
                        context.byte - b'0'
                    } else if b'@' < context.byte && context.byte < b'G' {
                        context.byte - 0x37
                    } else if b'`' < context.byte && context.byte < b'g' {
                        context.byte - 0x57
                    } else {
                        return Err(QueryError::Value(context.byte));
                    } as u8;

                    value.push(byte);
                } else {
                    if bs_has_bytes!(context, 1) {
                        bs_next!(context);
                    }

                    return Err(QueryError::Value(context.byte));
                }
            } else if context.byte == b'+' {
                value.push(b' ');
            } else {
                segment_fn(QuerySegment::FieldValue(&name, &value));

                name.clear();
                value.clear();

                break;
            }
        }
    }
}
