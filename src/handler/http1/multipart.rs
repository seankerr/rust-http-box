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
//! form data.

use field::{ FieldMap,
             FieldValue };
use http1::Http1Handler;
use util;
use util::FieldSegment;

use std;
use std::collections::HashMap;
use std::io::Result;
use std::fs::File;
use std::str;

/// Content disposition.
enum ContentDisposition {
    /// Unknown content disposition.
    Unknown,

    /// Field with value content disposition.
    Field,

    /// Field with file data content disposition.
    File(Vec<u8>, File)
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
///                     Content-Length: 7\r\n\
///                     \r\n\
///                     value 1\r\n\
///                     --ExampleBoundary\r\n\
///                     Content-Disposition: form-data; name=\"field1\"\r\n\
///                     Content-Length: 7\r\n\
///                     \r\n\
///                     value 2\r\n\
///                     --ExampleBoundary\r\n\
///                     Content-Disposition: form-data; name=\"field2\"\r\n\
///                     Content-Length: 7\r\n\
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

    /// Files.
    files: HashMap<String, File>,

    /// Indicates that parsing has finished.
    finished: bool,

    /// Current multipart section headers.
    headers: HashMap<String, String>,

    /// Maximum field size.
    max_field_size: usize,

    /// Maximum file size.
    max_file_size: usize,

    /// Field/value toggle.
    toggle: bool,

    /// File upload path.
    upload_path: String,

    /// Value buffer.
    value_buffer: Vec<u8>
}

impl MultipartHandler {
    /// Create a new `MultipartHandler` using default settings.
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
            files:               HashMap::with_capacity(0),
            finished:            false,
            max_field_size:      std::usize::MAX,
            max_file_size:       std::usize::MAX,
            headers:             HashMap::with_capacity(1),
            toggle:              false,
            upload_path:         "/tmp".to_string(),
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

    /// Retrieve `file` from the collection of files.
    pub fn file(&self, file: &str) -> Option<&File> {
        if let Some(file) = self.files.get(file) {
            Some(&file)
        } else {
            None
        }
    }

    /// Retrieve the files.
    pub fn files(&self) -> &HashMap<String, File> {
        &self.files
    }

    /// Flush the most recent field or file.
    fn flush_field_file(&mut self) {
        match self.content_disposition {
            ContentDisposition::Field => {
                if !self.field_buffer.is_empty() {
                    unsafe { self.fields.push_slice(&self.field_buffer, &self.value_buffer); }
                }
            },
            ContentDisposition::File(ref filename, ref file) => {
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

    /// Close a file.
    fn fn_close(file: &mut File) -> Result<()> {
        panic!("X");
    }

    /// Create a file.
    fn fn_create(filename: &[u8]) -> Result<File> {
        panic!("X");
    }

    /// Delete a file.
    fn fn_delete(file: &mut File) -> Result<()> {
        panic!("X");
    }

    /// Write to a file.
    fn fn_write(file: &mut File, data: &[u8]) -> Result<()> {
        panic!("X");
    }

    /// Indicates that `field` exists within the collection of fields.
    pub fn has_field(&self, field: &str) -> bool {
        self.fields.has_field(field)
    }

    /// Indicates that `file` exists within the collection of files.
    pub fn has_file(&self, file: &str) -> bool {
        self.files.contains_key(file)
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

    /// Set the max field size.
    pub fn set_max_field_size(&mut self, size: usize) {
        self.max_field_size = size;
    }

    /// Set the max file size.
    pub fn set_max_file_size(&mut self, size: usize) {
        self.max_file_size = size;
    }

    /// Set the file upload path.
    pub fn set_upload_path(&mut self, path: &str) {
        self.upload_path = path.to_string();
    }
}

impl Http1Handler for MultipartHandler {
    fn content_length(&mut self) -> Option<usize> {
        if let Some(content_length) = self.header_as_bytes("content-length") {
            let mut length: usize = 0;

            for byte in content_length.iter() {
                if is_digit!(*byte) {
                    if let Some(num) = length.checked_mul(10) {
                        if let Some(num) = num.checked_add((*byte - b'0') as usize) {
                            length = num;
                        } else {
                            return None;
                        }
                    } else {
                        return None;
                    }

                } else {
                    // contains non-digit
                    return None;
                }
            }

            Some(length)
        } else {
            None
        }
    }

    fn multipart_boundary(&mut self) -> Option<&[u8]> {
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
                              b';', true,
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

                    true
                }
            );

            if let Some(filename) = filename {
                if let Some(name) = name {
                    self.field_buffer.extend_from_slice(&name);

                    ContentDisposition::Unknown//File(filename, self.fn_create())
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
        self.headers.clear();

        true
    }

    fn on_multipart_data(&mut self, data: &[u8]) -> bool {
        match self.content_disposition {
            ContentDisposition::Field => {
                self.value_buffer.extend_from_slice(data);
            },
            ContentDisposition::File(ref filename, ref file) => {
            },
            ContentDisposition::Unknown => {
                // nothing to do
            }
        }

        true
    }
}
