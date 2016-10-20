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
//! URL encoded data.

use field::{ FieldMap,
             FieldValue };
use http1::Http1Handler;

/// `UrlEncodedHandler` is a suitable handler for the following parser functions:
///
/// - [`Parser::parse_url_encoded()`](../http1/struct.Parser.html#method.parse_url_encoded)
///
/// # Examples
///
/// ```
/// use http_box::UrlEncodedHandler;
/// use http_box::http1::Parser;
///
/// let mut h = UrlEncodedHandler::new();
/// let mut p = Parser::new();
///
/// p.parse_url_encoded(&mut h,
///                     b"Field1=Value%201&Field2=Value%202&Field1=Value%203",
///                     50);
///
/// assert_eq!("Value 1", h.field("Field1").unwrap().first().unwrap());
/// assert_eq!("Value 2", h.field("Field2").unwrap().first().unwrap());
/// assert_eq!("Value 3", h.field("Field1").unwrap().get(1).unwrap());
/// ```
#[derive(Default)]
pub struct UrlEncodedHandler {
    /// Field buffer.
    field_buffer: Vec<u8>,

    /// Fields.
    fields: FieldMap,

    /// Indicates that parsing has finished.
    finished: bool,

    /// Field/value toggle.
    toggle: bool,

    /// Value buffer.
    value_buffer: Vec<u8>,
}

impl UrlEncodedHandler {
    /// Create a new `UrlEncodedHandler`.
    pub fn new() -> UrlEncodedHandler {
        UrlEncodedHandler {
            field_buffer: Vec::new(),
            fields:       FieldMap::new(),
            finished:     false,
            toggle:       false,
            value_buffer: Vec::new()
        }
    }

    /// Retrieve `field` from the collection of fields.
    pub fn field<T: AsRef<str>>(&self, field: T) -> Option<&FieldValue> {
        self.fields.field(field)
    }

    /// Retrieve the fields.
    pub fn fields(&self) -> &FieldMap {
        &self.fields
    }

    /// Flush the most recent field/value.
    fn flush(&mut self) {
        if !self.field_buffer.is_empty() {
            unsafe { self.fields.push_slice(&self.field_buffer, &self.value_buffer); }
        }

        self.field_buffer.clear();
        self.value_buffer.clear();
    }

    /// Indicates that `field` exists within the collection of fields.
    pub fn has_field<T: AsRef<str>>(&self, field: T) -> bool {
        self.fields.has_field(field)
    }

    /// Indicates that parsing has finished.
    pub fn is_finished(&self) -> bool {
        self.finished
    }

    /// Reset the handler back to its original state.
    pub fn reset(&mut self) {
        self.finished = false;
        self.toggle   = false;

        self.field_buffer.clear();
        self.fields.clear();
        self.value_buffer.clear();
    }
}

impl Http1Handler for UrlEncodedHandler {
    fn on_body_finished(&mut self) -> bool {
        self.flush();

        self.finished = true;
        true
    }

    fn on_url_encoded_field(&mut self, field: &[u8]) -> bool {
        if self.toggle {
            self.flush();

            self.toggle = false;
        }

        self.field_buffer.extend_from_slice(field);
        true
    }

    fn on_url_encoded_value(&mut self, value: &[u8]) -> bool {
        self.value_buffer.extend_from_slice(value);

        self.toggle = true;
        true
    }
}
