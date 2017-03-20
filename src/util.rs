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

//! Utility functions for handling encoded data, query strings, and header fields.

use byte::is_token;
use byte_slice::ByteStream;

use std::{ fmt, str };

// -------------------------------------------------------------------------------------------------

macro_rules! submit_error {
    ($iter:expr, $error:expr) => ({
        bs_jump!($iter.context, bs_available!($iter.context));

        (*$iter.on_error)($error($iter.context.byte));

        return None;
    });
}

macro_rules! submit_name {
    ($iter:expr) => ({
        return Some((
            unsafe {
                let mut s = String::with_capacity($iter.name.len());

                s.as_mut_vec().extend_from_slice(&$iter.name);
                s
            },
            None
        ));
    });
}
macro_rules! submit_name_value {
    ($name:expr, $value:expr) => ({
        return Some((
            unsafe {
                let mut s = String::with_capacity($name.len());

                s.as_mut_vec().extend_from_slice(&$name);
                s
            },
            unsafe {
                let mut s = String::with_capacity($value.len());

                s.as_mut_vec().extend_from_slice(&$value);
                Some(s)
            }
        ));
    });

    ($iter:expr) => ({
        return Some((
            unsafe {
                let mut s = String::with_capacity($iter.name.len());

                s.as_mut_vec().extend_from_slice(&$iter.name);
                s
            },
            unsafe {
                let mut s = String::with_capacity($iter.value.len());

                s.as_mut_vec().extend_from_slice(&$iter.value);
                Some(s)
            }
        ));
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
                write!(
                    formatter,
                    "DecodeError::Byte(Invalid byte on byte {})",
                    x
                )
            },
            DecodeError::HexSequence(x) => {
                write!(
                    formatter,
                    "DecodeError::HexSequence(Invalid hex sequence on byte {})",
                    x
                )
            }
        }
    }
}

impl fmt::Display for DecodeError {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            DecodeError::Byte(x) => {
                write!(
                    formatter,
                    "Invalid byte on byte {}",
                    x
                )
            },
            DecodeError::HexSequence(x) => {
                write!(
                    formatter,
                    "Invalid hex sequence on byte {}",
                    x
                )
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
                write!(
                    formatter,
                    "FieldError::Name(Invalid field name on byte {})",
                    x
                )
            },
            FieldError::Value(x) => {
                write!(
                    formatter,
                    "FieldError::Value(Invalid field value on byte {})",
                    x
                )
            }
        }
    }
}

impl fmt::Display for FieldError {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            FieldError::Name(x) => {
                write!(
                    formatter,
                    "Invalid field name on byte {}",
                    x
                )
            },
            FieldError::Value(x) => {
                write!(
                    formatter,
                    "Invalid field value on byte {}",
                    x
                )
            }
        }
    }
}

// -------------------------------------------------------------------------------------------------

/// Header field iterator.
///
/// This allows you to iterate over a header field to retrieve `(name, value)` pairs.
///
/// # Errors
///
/// - [`FieldError::Name`](enum.FieldError.html#variant.Name)
/// - [`FieldError::Value`](enum.FieldError.html#variant.Value)
///
/// ```rust
/// extern crate http_box;
///
/// use http_box::util::{ FieldError, FieldIterator };
///
/// fn main() {
///     // notice the upper-cased parameter names that are normalized to lower-case thanks
///     // to the third parameter being `true`
///     let field = b"COMPRESSION=bzip; BOUNDARY=\"longrandomboundarystring\"";
///
///     for (n, (name, value)) in FieldIterator::new(field, b';', true).enumerate() {
///         if n == 0 {
///             assert_eq!(
///                 name,
///                 "compression"
///             );
///
///             assert_eq!(
///                 value.unwrap(),
///                 "bzip"
///             );
///         } else if n == 1 {
///             assert_eq!(
///                 name,
///                 "boundary"
///             );
///
///             assert_eq!(
///                 value.unwrap(),
///                 "longrandomboundarystring"
///             );
///         }
///     }
/// }
/// ```
pub struct FieldIterator<'a> {
    context:   ByteStream<'a>,
    delimiter: u8,
    name:      Vec<u8>,
    normalize: bool,
    on_error:  Box<FnMut(FieldError) + 'a>,
    value:     Vec<u8>
}

impl<'a> FieldIterator<'a> {
    /// Create a new `FieldIterator`.
    ///
    /// # Arguments
    ///
    /// **`field`**
    ///
    /// The header field.
    ///
    /// **`delimiter`**
    ///
    /// The field delimiter.
    ///
    /// **`normalize`**
    ///
    /// Indicates that field names should be normalized to lower-case.
    pub fn new(field: &'a [u8], delimiter: u8, normalize: bool) -> FieldIterator<'a> {
        FieldIterator{
            context:   ByteStream::new(field),
            delimiter: delimiter,
            name:      Vec::new(),
            normalize: normalize,
            on_error:  Box::new(|_|{}),
            value:     Vec::new()
        }
    }

    /// Set the on error callback.
    ///
    /// # Arguments
    ///
    /// **`on_error`**
    ///
    /// The callback.
    pub fn on_error<F>(&mut self, on_error: F) -> &mut Self
    where F : FnMut(FieldError) + 'a {
        self.on_error = Box::new(on_error);
        self
    }
}

impl<'a> Iterator for FieldIterator<'a> {
    type Item = (String, Option<String>);

    fn next(&mut self) -> Option<(String, Option<String>)> {
        if bs_available!(self.context) == 0 {
            return None;
        }

        self.name.clear();
        self.value.clear();

        loop {
            // parsing name
            consume_spaces!(
                self.context,

                // on end-of-stream
                return None
            );

            bs_mark!(self.context);

            collect_tokens_iter!(
                self,
                self.context,
                FieldError::Name,

                // stop on these bytes
                   self.context.byte == b'='
                || self.context.byte == self.delimiter
                || self.context.byte == b'/'
                || (self.normalize && self.context.byte > 0x40 && self.context.byte < 0x5B),

                // on end-of-stream
                {
                    // name without a value
                    if bs_slice_length!(self.context) > 0 {
                        self.name.extend_from_slice(bs_slice!(self.context));
                    }

                    submit_name!(self);
                }
            );

            self.name.extend_from_slice(bs_slice_ignore!(self.context));

            match self.context.byte {
                b'=' => {
                    // parsing value
                    if bs_available!(self.context) == 0 {
                        // name without a value
                        submit_name!(self);
                    }

                    bs_next!(self.context);

                    if self.context.byte == b'"' {
                        // quoted value
                        loop {
                            bs_mark!(self.context);

                            collect_quoted_iter!(
                                self,
                                self.context,
                                FieldError::Value,

                                // on end-of-stream
                                // didn't find an ending quote
                                submit_error!(self, FieldError::Value)
                            );

                            if self.context.byte == b'"' {
                                // found end quote
                                self.value.extend_from_slice(bs_slice_ignore!(self.context));

                                consume_spaces!(
                                    self.context,

                                    // on end-of-stream
                                    submit_name_value!(self)
                                );

                                if bs_available!(self.context) == 0 {
                                    submit_name_value!(self);
                                }

                                bs_next!(self.context);

                                if self.context.byte == self.delimiter {
                                    submit_name_value!(self);
                                }

                                // expected a semicolon to end the value
                                submit_error!(self, FieldError::Value);
                            } else {
                                // found backslash
                                if bs_is_eos!(self.context) {
                                    submit_error!(self, FieldError::Name);
                                }

                                self.value.extend_from_slice(bs_slice_ignore!(self.context));

                                bs_next!(self.context);

                                self.value.push(self.context.byte);
                            }
                        }
                    } else {
                        // unquoted value
                        bs_replay!(self.context);
                        bs_mark!(self.context);

                        collect_field_iter!(
                            self,
                            self.context,
                            FieldError::Value,

                            // stop on these bytes
                            self.context.byte == self.delimiter,

                            // on end-of-stream
                            {
                                if bs_slice_length!(self.context) > 0 {
                                    self.value.extend_from_slice(bs_slice!(self.context));
                                }

                                submit_name_value!(self);
                            }
                        );

                        if bs_slice_length!(self.context) == 0 {
                            // name without a value
                            submit_name!(self);
                        }

                        submit_name_value!(self.name, bs_slice_ignore!(self.context));
                    }
                },
                b'/' => {
                    // this isn't allowed as a token, but since it's a name-only field, it's allowed
                    self.name.push(b'/');
                },
                byte if byte == self.delimiter => {
                    // name without a value
                    submit_name!(self);
                },
                _ => {
                    // upper-cased byte, let's lower-case it
                    self.name.push(self.context.byte + 0x20);
                }
            }
        }
    }
}

// -------------------------------------------------------------------------------------------------

/// Query errors.
pub enum QueryError {
    /// Invalid query name.
    Name(u8),

    /// Invalid query value.
    Value(u8)
}

impl fmt::Debug for QueryError {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            QueryError::Name(x) => {
                write!(
                    formatter,
                    "QueryError::Name(Invalid query name on byte {})",
                    x
                )
            },
            QueryError::Value(x) => {
                write!(
                    formatter,
                    "QueryError::Value(Invalid query value on byte {})",
                    x
                )
            }
        }
    }
}

impl fmt::Display for QueryError {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            QueryError::Name(x) => {
                write!(
                    formatter,
                    "Invalid query name on byte {}",
                    x
                )
            },
            QueryError::Value(x) => {
                write!(
                    formatter,
                    "Invalid query value on byte {}",
                    x
                )
            }
        }
    }
}

// -------------------------------------------------------------------------------------------------

/// Query iterator.
///
/// This allows you to iterate over a query string to retrieve `(name, value)` pairs.
///
/// # Errors
///
/// - [`QueryError::Name`](enum.QueryError.html#variant.Name)
/// - [`QueryError::Value`](enum.QueryError.html#variant.Value)
///
/// ```rust
/// extern crate http_box;
///
/// use http_box::util::QueryIterator;
///
/// fn main() {
///     let query = b"field1=value1&field2=value2&field3";
///
///     for (n, (name, value)) in QueryIterator::new(query).enumerate() {
///         if n == 0 {
///             assert_eq!(
///                 name,
///                 "field1"
///             );
///
///             assert_eq!(
///                 value.unwrap(),
///                 "value1"
///             );
///         } else if n == 1 {
///             assert_eq!(
///                 name,
///                 "field2"
///             );
///
///             assert_eq!(
///                 value.unwrap(),
///                 "value2"
///             );
///         } else if n == 2 {
///             assert_eq!(
///                 name,
///                 "field3"
///             );
///
///             assert_eq!(
///                 value,
///                 None
///             );
///         }
///     }
/// }
/// ```
pub struct QueryIterator<'a> {
    context:  ByteStream<'a>,
    name:     Vec<u8>,
    on_error: Box<FnMut(QueryError) + 'a>,
    value:    Vec<u8>
}

impl<'a> QueryIterator<'a> {
    /// Create a new `QueryIterator`.
    ///
    /// # Arguments
    ///
    /// **`query`**
    ///
    /// The query string.
    pub fn new(query: &'a [u8]) -> QueryIterator<'a> {
        QueryIterator{
            context:  ByteStream::new(query),
            name:     Vec::new(),
            on_error: Box::new(|_|{}),
            value:    Vec::new()
        }
    }

    /// Set the on error callback.
    ///
    /// # Arguments
    ///
    /// **`on_error`**
    ///
    /// The callback.
    pub fn on_error<F>(&mut self, on_error: F) -> &mut Self
    where F : FnMut(QueryError) + 'a {
        self.on_error = Box::new(on_error);
        self
    }
}

impl<'a> Iterator for QueryIterator<'a> {
    type Item = (String, Option<String>);

    fn next(&mut self) -> Option<(String, Option<String>)> {
        if bs_available!(self.context) == 0 {
            return None;
        }

        self.name.clear();
        self.value.clear();

        loop {
            // field loop
            loop {
                bs_mark!(self.context);

                collect_visible_iter!(
                    self,
                    self.context,
                    QueryError::Name,

                    // stop on these bytes
                       self.context.byte == b'%'
                    || self.context.byte == b'+'
                    || self.context.byte == b'='
                    || self.context.byte == b'&'
                    || self.context.byte == b';',

                    // on end-of-stream
                    {
                        if bs_slice_length!(self.context) > 0 {
                            self.name.extend_from_slice(bs_slice!(self.context));
                        }

                        submit_name!(self);
                    }
                );

                if bs_slice_length!(self.context) > 1 {
                    self.name.extend_from_slice(bs_slice_ignore!(self.context));
                }

                match self.context.byte {
                    b'%' => {
                        if bs_has_bytes!(self.context, 2) {
                            self.name.push(collect_hex8_iter!(
                                self,
                                self.context,
                                QueryError::Name
                            ));
                        } else {
                            if bs_has_bytes!(self.context, 1) {
                                bs_next!(self.context);
                            }

                            submit_error!(self, QueryError::Name);
                        }
                    },
                    b'+' => {
                        self.name.push(b' ');
                    },
                    b'=' => {
                        if self.context.stream_index == 1 {
                            // first byte cannot be an equal sign
                            submit_error!(self, QueryError::Name);
                        }

                        break;
                    },
                    _ if self.context.stream_index == 1 => {
                        // first byte cannot be a delimiter
                        submit_error!(self, QueryError::Name);
                    },
                    _ => {
                        // name without a value
                        submit_name!(self);
                    }
                }
            }

            // value loop
            loop {
                bs_mark!(self.context);

                collect_visible_iter!(
                    self,
                    self.context,
                    QueryError::Value,

                    // stop on these bytes
                       self.context.byte == b'%'
                    || self.context.byte == b'+'
                    || self.context.byte == b'&'
                    || self.context.byte == b';',

                    // on end-of-stream
                    {
                        if bs_slice_length!(self.context) > 0 {
                            self.value.extend_from_slice(bs_slice!(self.context));
                        }

                        submit_name_value!(self);
                    }
                );

                if bs_slice_length!(self.context) > 1 {
                    self.value.extend_from_slice(bs_slice_ignore!(self.context));
                }

                match self.context.byte {
                    b'%' => {
                        if bs_has_bytes!(self.context, 2) {
                            self.value.push(collect_hex8_iter!(
                                self,
                                self.context,
                                QueryError::Value
                            ));
                        } else {
                            if bs_has_bytes!(self.context, 1) {
                                bs_next!(self.context);
                            }

                            submit_error!(self, QueryError::Value);
                        }
                    },
                    b'+' => {
                        self.value.push(b' ');
                    },
                    _ => {
                        // name with a value
                        submit_name_value!(self);
                    }
                }
            }
        }
    }
}

// -------------------------------------------------------------------------------------------------

/// Decode URL encoded data.
///
/// # Arguments
///
/// **`encoded`**
///
/// The encoded data.
///
/// # Returns
///
/// **`String`**
///
/// The decoded string.
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
/// let string = match util::decode(b"fancy%20url%20encoded%20data") {
///     Ok(string) => string,
///     Err(_) => panic!()
/// };
///
/// assert_eq!(string, "fancy url encoded data");
/// ```
pub fn decode(encoded: &[u8]) -> Result<String, DecodeError> {
    macro_rules! submit {
        ($string:expr, $slice:expr) => (unsafe {
            $string.as_mut_vec().extend_from_slice($slice);
        });
    }

    let mut context = ByteStream::new(encoded);
    let mut string  = String::new();

    loop {
        bs_mark!(context);

        collect_visible!(
            context,
            DecodeError::Byte,

            // stop on these bytes
               context.byte == b'%'
            || context.byte == b'+',

            // on end-of-stream
            {
                if context.mark_index < context.stream_index {
                    submit!(string, bs_slice!(context));
                }

                return Ok(string);
            }
        );

        if bs_slice_length!(context) > 1 {
            submit!(string, bs_slice_ignore!(context));
        }

        if context.byte == b'+' {
            submit!(string, b" ");
        } else if bs_has_bytes!(context, 2) {
            submit!(string, &[
                collect_hex8!(context, DecodeError::HexSequence)
            ]);
        } else {
            if bs_has_bytes!(context, 1) {
                bs_next!(context);
            }

            return Err(DecodeError::HexSequence(context.byte));
        }
    }
}
