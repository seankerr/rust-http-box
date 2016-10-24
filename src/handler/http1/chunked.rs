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

//! [`Http1Handler`](../../../http1/trait.Http1Handler.html) implementation for processing
//! chunk encoded data.

use http1::Http1Handler;

use std::collections::HashMap;

/// `ChunkedHandler` is a suitable handler for the following parser functions:
///
/// - [`Parser::parse_chunked()`](../http1/struct.Parser.html#method.parse_chunked)
///
/// # Examples
///
/// ```
/// use http_box::ChunkedHandler;
/// use http_box::http1::Parser;
///
/// let mut chunk_data = vec![];
///
/// {
///     let mut h = ChunkedHandler::new(|h,s| {
///         // h = chunk handler
///         // s = slice of raw data
///         assert_eq!(false, h.is_finished());
///         assert_eq!(0, h.index());
///         assert_eq!(4, h.len());
///         assert_eq!(b"data", s);
///         assert_eq!("value", h.extension("extension").unwrap());
///         chunk_data.extend_from_slice(s);
///         true
///     });
///
///     let mut p = Parser::new();
///
///     p.parse_chunked(&mut h,
///                     b"4;extension=value\r\n\
///                       data\r\n\
///                       0\r\n\
///                       Trailer: value\r\n\
///                       \r\n");
///
///     assert!(h.is_finished());
///     assert_eq!(1, h.index());
///     assert_eq!(0, h.len());
///     assert_eq!("value", h.trailer("trailer").unwrap());
/// }
///
/// assert_eq!(b"data", &chunk_data[..]);
/// ```
pub struct ChunkedHandler<F> where F : FnMut(&mut ChunkedHandler<F>, &[u8]) -> bool {
    /// Data callback.
    data_fn: Option<F>,

    /// Extensions for the current chunk.
    extensions: HashMap<String, String>,

    /// Extension name buffer and trailer field buffer.
    field_buffer: Vec<u8>,

    /// Indicates that chunk parsing has finished.
    finished: bool,

    /// The current chunk index.
    index: u32,

    /// Current chunk length.
    length: usize,

    /// Extension name/value, and trailer field/value toggle.
    toggle: bool,

    /// Trailers.
    trailers: HashMap<String, String>,

    /// Extension value buffer and trailer value buffer.
    value_buffer: Vec<u8>
}

impl<F> ChunkedHandler<F> where F : FnMut(&mut ChunkedHandler<F>, &[u8]) -> bool {
    /// Create a new `ChunkedHandler`.
    ///
    /// # Arguments
    ///
    /// **`data_fn`**
    ///
    /// A closure that receives the `&mut ChunkedHandler`, and the current chunk of data.
    pub fn new(data_fn: F) -> ChunkedHandler<F> {
        ChunkedHandler{
            data_fn:      Some(data_fn),
            extensions:   HashMap::new(),
            field_buffer: Vec::new(),
            finished:     false,
            index:        0,
            length:       0,
            toggle:       false,
            value_buffer: Vec::new(),
            trailers:     HashMap::new(),
        }
    }

    /// Flush the most recent extension name/value.
    fn flush_extension(&mut self) {
        if !self.field_buffer.is_empty() {
            self.extensions.insert(
                // name
                unsafe {
                    let mut s = String::with_capacity(self.field_buffer.len());

                    s.as_mut_vec().extend_from_slice(&self.field_buffer);
                    s
                },

                // value
                unsafe {
                    let mut s = String::with_capacity(self.value_buffer.len());

                    s.as_mut_vec().extend_from_slice(&self.value_buffer);
                    s
                }
            );
        }

        self.field_buffer.clear();
        self.value_buffer.clear();
    }

    /// Flush the most recent trailer field/value.
    fn flush_trailer(&mut self) {
        if !self.field_buffer.is_empty() {
            self.trailers.insert(
                // name
                unsafe {
                    let mut s = String::with_capacity(self.field_buffer.len());

                    s.as_mut_vec().extend_from_slice(&self.field_buffer);
                    s
                },

                // value
                unsafe {
                    let mut s = String::with_capacity(self.value_buffer.len());

                    s.as_mut_vec().extend_from_slice(&self.value_buffer);
                    s
                }
            );
        }

        self.field_buffer.clear();
        self.value_buffer.clear();
    }

    /// Retrieve `extension` from the collection of extensions.
    pub fn extension<T: AsRef<str>>(&self, extension: T) -> Option<&str> {
        if let Some(ref extension) = self.extensions.get(extension.as_ref()) {
            Some(&extension[..])
        } else {
            None
        }
    }

    /// Retrieve the extensions for the current chunk.
    pub fn extensions(&self) -> &HashMap<String, String> {
        &self.extensions
    }

    /// Indicates that `extension` exists within the collection of extensions.
    pub fn has_extension<T: AsRef<str>>(&self, extension: T) -> bool {
        self.extensions.contains_key(extension.as_ref())
    }

    /// Indicates that `trailer` exists within the collection of trailers.
    pub fn has_trailer<T: AsRef<str>>(&self, trailer: T) -> bool {
        self.trailers.contains_key(trailer.as_ref())
    }

    /// Retrieve the current chunk index.
    pub fn index(&self) -> u32 {
        self.index - 1
    }

    /// Indicates that chunk parsing has finished.
    pub fn is_finished(&self) -> bool {
        self.finished
    }

    /// Retrieve the current chunk length.
    pub fn len(&self) -> usize {
        self.length
    }

    /// Retrieve `trailer` from the collection of trailers.
    pub fn trailer<T: AsRef<str>>(&self, trailer: T) -> Option<&str> {
        if let Some(ref trailer) = self.trailers.get(trailer.as_ref()) {
            Some(&trailer[..])
        } else {
            None
        }
    }

    /// Retrieve the trailers.
    pub fn trailers(&self) -> &HashMap<String, String> {
        &self.trailers
    }

    /// Reset the handler to its original state.
    pub fn reset(&mut self) {
        self.finished = false;
        self.index    = 0;
        self.length   = 0;
        self.toggle   = false;

        self.extensions.clear();
        self.field_buffer.clear();
        self.trailers.clear();
        self.value_buffer.clear();
    }
}

impl<F> Http1Handler for ChunkedHandler<F> where F : FnMut(&mut ChunkedHandler<F>, &[u8]) -> bool {
    fn on_body_finished(&mut self) -> bool {
        self.finished = true;
        true
    }

    fn on_chunk_data(&mut self, data: &[u8]) -> bool {
        if let Some(mut data_fn) = self.data_fn.take() {
            if data_fn(self, data) {
                self.data_fn = Some(data_fn);
                true
            } else {
                self.data_fn = Some(data_fn);
                false
            }
        } else {
            // this should never happen
            panic!();
        }
    }

    fn on_chunk_extension_name(&mut self, name: &[u8]) -> bool {
        if self.toggle {
            self.flush_extension();

            self.toggle = false;
        }

        self.field_buffer.extend_from_slice(name);

        true
    }

    fn on_chunk_extension_value(&mut self, value: &[u8]) -> bool {
        self.value_buffer.extend_from_slice(value);

        self.toggle = true;
        true
    }

    fn on_chunk_extensions_finished(&mut self) -> bool {
        if !self.field_buffer.is_empty() {
            self.flush_extension();
        }

        true
    }

    fn on_chunk_length(&mut self, length: usize) -> bool {
        self.extensions.clear();
        self.field_buffer.clear();
        self.value_buffer.clear();

        self.index  += 1;
        self.length  = length;
        true
    }

    fn on_header_field(&mut self, field: &[u8]) -> bool {
        if self.toggle {
            self.flush_trailer();

            self.toggle = false;
        }

        self.field_buffer.extend_from_slice(field);

        true
    }

    fn on_header_value(&mut self, value: &[u8]) -> bool {
        self.value_buffer.extend_from_slice(value);

        self.toggle = true;
        true
    }

    fn on_headers_finished(&mut self) -> bool {
        self.flush_trailer();
        true
    }
}
