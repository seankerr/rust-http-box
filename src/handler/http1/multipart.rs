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

use std::collections::HashMap;

pub struct MultipartHandler {
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
    pub fn new() -> MultipartHandler {
        MultipartHandler {
            field_buffer: Vec::new(),
            fields:       FieldMap::new(),
            finished:     false,
            headers:      HashMap::new(),
            toggle:       false,
            value_buffer: Vec::new()
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

    /// Flush the most recent field/value.
    fn flush_field(&mut self) {
        if !self.field_buffer.is_empty() {
            unsafe { self.fields.push_slice(&self.field_buffer, &self.value_buffer); }
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

impl Http1Handler for MultipartHandler {
}
