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

use byte::is_token;

use byte_slice::ByteStream;
use std::fmt;

/// Field errors.
pub enum FieldError {
    /// Invalid field name.
    Name(u8),

    /// Invalid field value.
    Value(u8)
}

impl FieldError {
    /// Format this for debug and display purposes.
    fn format(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            FieldError::Name(x) => {
                write!(
                    formatter,
                    "<FieldError::Name: Invalid field name on byte {}>",
                    x
                )
            },
            FieldError::Value(x) => {
                write!(
                    formatter,
                    "<FieldError::Value: Invalid field value on byte {}>",
                    x
                )
            }
        }
    }
}

impl fmt::Debug for FieldError {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        self.format(formatter)
    }
}

impl fmt::Display for FieldError {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        self.format(formatter)
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
