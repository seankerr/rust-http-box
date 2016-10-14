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

//! [`Http1Handler`](../../../http1/trait.Http1Handler.html) implementation for processing multipart
//! data.

use field::{ FieldMap,
             FieldValue };
use http1::Http1Handler;
use util;
use util::FieldSegment;

use std::collections::HashMap;
use std::str;

/// Content disposition.
enum ContentDisposition {
    /// Unknown content disposition.
    Unknown,

    /// Field with value content disposition.
    Field,

    /// Field with file data content disposition.
    File(Vec<u8>)
}

// -------------------------------------------------------------------------------------------------

/// `MultipartHandler` is a suitable handler for the following parser functions:
///
/// - [`Parser::parse_multipart()`](../http1/struct.Parser.html#method.parse_multipart)
///
/// # Examples
///
/// ```
/// use http_box::MultipartHandler;
/// use http_box::http1::Parser;
///
/// let mut h = MultipartHandler::new(b"ExampleBoundary");
/// let mut p = Parser::new();
///
/// p.parse_multipart(&mut h,
///                   b"--ExampleBoundary\r\n\
///                     Content-Disposition: form-data; name=\"field1\"\r\n\
///                     \r\n\
///                     value 1\r\n\
///                     --ExampleBoundary\r\n\
///                     Content-Disposition: form-data; name=\"field1\"\r\n\
///                     \r\n\
///                     value 2\r\n\
///                     --ExampleBoundary\r\n\
///                     Content-Disposition: form-data; name=\"field2\"\r\n\
///                     \r\n\
///                     value 3\r\n\
///                     --ExampleBoundary--\r\n");
///
/// assert_eq!("value 1", h.field("field1").unwrap().get(0).unwrap());
/// assert_eq!("value 2", h.field("field1").unwrap().get(1).unwrap());
/// assert_eq!("value 3", h.field("field2").unwrap().first().unwrap());
/// assert!(h.is_finished());
/// ```
pub struct MultipartHandler {
    /// Boundary.
    boundary: Vec<u8>,

    /// Current content disposition.
    content_disposition: ContentDisposition,

    /// Field buffer.
    field_buffer: Vec<u8>,

    /// Fields.
    fields: FieldMap,

    /// Indicates that parsing has finished.
    finished: bool,

    /// Current multipart section headers.
    headers: HashMap<String, String>,

    /// Field/value toggle.
    toggle: bool,

    /// Value buffer.
    value_buffer: Vec<u8>
}

impl MultipartHandler {
    /// Create a new `MultipartHandler`.
    ///
    /// # Arguments
    ///
    /// **`boundary`**
    ///
    /// The multipart boundary.
    pub fn new(boundary: &[u8]) -> MultipartHandler {
        MultipartHandler {
            boundary: {
                let mut v = Vec::with_capacity(boundary.len());

                v.extend_from_slice(boundary);
                v
            },
            content_disposition: ContentDisposition::Unknown,
            field_buffer:        Vec::new(),
            fields:              FieldMap::new(),
            finished:            false,
            headers:             HashMap::new(),
            toggle:              false,
            value_buffer:        Vec::new()
        }
    }

    /// Retrieve `field` from the collection of fields.
    pub fn field(&self, field: &str) -> Option<&FieldValue> {
        self.fields.field(field)
    }

    /// Retrieve the fields.
    pub fn fields(&self) -> &FieldMap {
        &self.fields
    }

    /// Flush the most recent field or file.
    fn flush_field_file(&mut self) {
        match self.content_disposition {
            ContentDisposition::Field => {
                if !self.field_buffer.is_empty() {
                    unsafe { self.fields.push_slice(&self.field_buffer, &self.value_buffer); }
                }
            },
            ContentDisposition::File(ref filename) => {
                // todo: flush file
            },
            ContentDisposition::Unknown => {
                // nothing to do
            }
        }

        self.field_buffer.clear();
        self.value_buffer.clear();
    }

    /// Flush the most recent header/value.
    fn flush_header(&mut self) {
        if !self.field_buffer.is_empty() {
            self.headers.insert(unsafe {
                let mut s = String::with_capacity(self.field_buffer.len());

                s.as_mut_vec().extend_from_slice(&self.field_buffer);
                s
            }, unsafe {
                let mut s = String::with_capacity(self.value_buffer.len());

                s.as_mut_vec().extend_from_slice(&self.value_buffer);
                s
            });
        }

        self.field_buffer.clear();
        self.value_buffer.clear();
    }

    /// Indicates that `field` exists within the collection of fields.
    pub fn has_field(&self, field: &str) -> bool {
        self.fields.has_field(field)
    }

    /// Indicates that `header` exists within the collection of headers.
    pub fn has_header(&self, header: &str) -> bool {
        self.headers.contains_key(header)
    }

    /// Retrieve `header` from the collection of headers.
    pub fn header(&self, header: &str) -> Option<&str> {
        if let Some(header) = self.headers.get(header) {
            Some(&header[..])
        } else {
            None
        }
    }

    /// Retrieve `header` as a slice of bytes from the collection of headers.
    pub fn header_as_bytes(&self, header: &str) -> Option<&[u8]> {
        if let Some(header) = self.headers.get(header) {
            Some(header.as_bytes())
        } else {
            None
        }
    }

    /// Retrieve the collection of headers.
    pub fn headers(&self) -> &HashMap<String, String> {
        &self.headers
    }

    /// Indicates that parsing has finished.
    pub fn is_finished(&self) -> bool {
        self.finished
    }

    /// Reset the handler back to its original state.
    pub fn reset(&mut self) {
        self.content_disposition = ContentDisposition::Unknown;
        self.finished            = false;
        self.toggle              = false;

        self.field_buffer.clear();
        self.fields.clear();
        self.headers.clear();
        self.value_buffer.clear();
    }
}

impl Http1Handler for MultipartHandler {
    fn get_multipart_boundary(&mut self) -> Option<&[u8]> {
        Some(&self.boundary)
    }

    fn on_body_finished(&mut self) -> bool {
        self.flush_field_file();

        self.finished = true;

        true
    }

    fn on_header_field(&mut self, field: &[u8]) -> bool {
        if self.toggle {
            self.flush_header();

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
        self.flush_header();

        self.content_disposition = if self.has_header("content-disposition") {
            let mut filename = None;
            let mut name     = None;

            util::parse_field(self.header_as_bytes("content-disposition").unwrap(),
                              b';',
                |s| {
                    match s {
                        FieldSegment::NameValue(n, v) => {
                            if n == b"name" {
                                let mut vec = Vec::with_capacity(v.len());

                                vec.extend_from_slice(v);

                                name = Some(vec);
                            } else if n == b"filename" {
                                let mut vec = Vec::with_capacity(v.len());

                                vec.extend_from_slice(v);

                                filename = Some(vec);
                            }
                        },
                        _ => {}
                    }
                }
            );

            if let Some(filename) = filename {
                if let Some(name) = name {
                    self.field_buffer.extend_from_slice(&name);

                    ContentDisposition::File(filename)
                } else {
                    ContentDisposition::Unknown
                }
            } else if let Some(name) = name {
                self.field_buffer.extend_from_slice(&name);

                ContentDisposition::Field
            } else {
                ContentDisposition::Unknown
            }
        } else {
            ContentDisposition::Unknown
        };

        true
    }

    fn on_multipart_begin(&mut self) -> bool {
        self.flush_field_file();

        true
    }

    fn on_multipart_data(&mut self, data: &[u8]) -> bool {
        match self.content_disposition {
            ContentDisposition::Field => {
                self.value_buffer.extend_from_slice(data);
            },
            ContentDisposition::File(ref filename) => {
            },
            ContentDisposition::Unknown => {
                // nothing to do
            }
        }

        true
    }
}
