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

use byte_slice::ByteStream;
use std::fmt;

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
                    "<QueryError::Name: Invalid query name on byte {}>",
                    x
                )
            },
            QueryError::Value(x) => {
                write!(
                    formatter,
                    "<QueryError::Value: Invalid query value on byte {}>",
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
