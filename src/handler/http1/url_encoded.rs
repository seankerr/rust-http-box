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

use http1::Http1Handler;

use std::collections::HashMap;

#[derive(Default)]
/// `UrlEncodedHandler` is a suitable handler for the following parser functions:
///
/// - [`Parser::parse_url_encoded()`](../http1/struct.Parser.html#method.parse_url_encoded)
///
/// # Example
///
/// ```
/// use http_box::UrlEncodedHandler;
/// use http_box::http1::Parser;
///
/// let mut h = UrlEncodedHandler::new();
/// let mut p = Parser::new();
///
/// p.parse_url_encoded(&mut h,
///                     b"Field1=Value%201&Field2=Value%202",
///                     33);
///
/// assert_eq!("Value 1", h.get_fields().get("Field1").unwrap());
/// assert_eq!("Value 2", h.get_fields().get("Field2").unwrap());
/// ```
pub struct UrlEncodedHandler {
    /// Field buffer.
    field_buffer: String,

    /// Map of all fields.
    fields: HashMap<String,String>,

    /// Indicates that the body is finished parsing.
    finished: bool,

    /// Field/value toggle.
    toggle: bool,

    /// Value buffer.
    value_buffer: String,
}

impl UrlEncodedHandler {
    /// Create a new `UrlEncodedHandler`.
    pub fn new() -> UrlEncodedHandler {
        UrlEncodedHandler {
            field_buffer: String::new(),
            fields:       HashMap::new(),
            finished:     false,
            toggle:       false,
            value_buffer: String::new()
        }
    }

    /// Flush the most recent field/value.
    fn flush(&mut self) {
        if self.field_buffer.len() > 0 {
            self.fields.insert(self.field_buffer.clone(), self.value_buffer.clone());
        }

        self.field_buffer.clear();
        self.value_buffer.clear();
    }

    /// Retrieve the fields.
    pub fn get_fields(&self) -> &HashMap<String,String> {
        &self.fields
    }

    /// Indicates that the body is finished parsing.
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

        unsafe {
            self.field_buffer
                .as_mut_vec()
                .extend_from_slice(field);
        }

        true
    }

    fn on_url_encoded_value(&mut self, value: &[u8]) -> bool {
        unsafe {
            self.value_buffer
                .as_mut_vec()
                .extend_from_slice(value);
        }

        self.toggle = true;
        true
    }
}
