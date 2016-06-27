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

use field::FieldValue;
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
/// //assert_eq!("Value 1", h.get_fields().get("Field1").unwrap().0);
/// //assert_eq!("Value 2", h.get_fields().get("Field2").unwrap().0);
/// ```
pub struct UrlEncodedHandler {
    /// Field buffer.
    field_buffer: String,

    /// Fields.
    fields: HashMap<String,FieldValue>,

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
        if !self.field_buffer.is_empty() {
            self.value_buffer.reserve(0);

            /*
            match self.fields.entry(self.field_buffer.clone()) {
                Entry::Vacant(mut entry) => {
                    entry.insert(FieldValue::Single(self.value_buffer.clone()));
                },
                Entry::Occupied(mut entry) => {
                    if entry.get().is_single() {
                        // single value that needs converted to multiple
                        let mut vec       = vec![];
                        let mut old_value = entry.insert(Fieldvec);
                    } else {
                        // multiple values
                        match entry.get_mut() {
                            &mut FieldValue::Multiple(ref mut vec) => {
                                vec.push(self.value_buffer.clone());
                            },
                            // this should never happen
                            _ => panic!()
                        }

                        &mut FieldValue::Multiple(ref mut v) => {
                            v.push(self.value_buffer.clone());
                        },
                        &mut FieldValue::Single(ref mut s) => {
                            // move old string pointer data into a new one,
                            // add it into a new vector, then set the vector
                            //
                            // once finished, forget the old string memory
                            unsafe {
                                let new = String::from_raw_parts(s.as_ptr() as *mut u8,
                                                                 s.len(),
                                                                 s.capacity());

                                let old = entry.insert(
                                              FieldValue::Multiple(
                                                  vec![new, self.value_buffer.clone()]
                                              )
                                          );

                                mem::forget(old);
                            }
                        }
                    }
                }
            }
            */
        }

        self.field_buffer.clear();
        self.value_buffer.clear();
    }

    /// Retrieve the fields.
    pub fn get_fields(&self) -> &HashMap<String,FieldValue> {
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
