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

use extra::parameter::{ ParameterMap,
                        ParameterValue };
use http1::Http1Handler;

use std::str;

/// `UrlEncodedHandler` is a suitable handler for the following parser functions:
///
/// - [`Parser::parse_url_encoded()`](../http1/struct.Parser.html#method.parse_url_encoded)
///
/// # Examples
///
/// ```
/// use http_box::extra::handler::UrlEncodedHandler;
/// use http_box::http1::Parser;
///
/// let mut h = UrlEncodedHandler::new();
/// let mut p = Parser::new();
///
/// p.parse_url_encoded(&mut h,
///                     b"Field1=Value%201&Field2=Value%202&Field1=Value%203",
///                     50);
///
/// assert_eq!("Value 1", h.parameter("Field1").unwrap().first().unwrap());
/// assert_eq!("Value 2", h.parameter("Field2").unwrap().first().unwrap());
/// assert_eq!("Value 3", h.parameter("Field1").unwrap().get(1).unwrap());
/// ```
#[derive(Default)]
pub struct UrlEncodedHandler {
    /// Field buffer.
    field_buffer: Vec<u8>,

    /// Indicates that parsing has finished.
    finished: bool,

    /// Parameters.
    parameters: ParameterMap,

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
            finished:     false,
            parameters:   ParameterMap::new(),
            toggle:       false,
            value_buffer: Vec::new()
        }
    }

    /// Flush the most recent field/value.
    fn flush(&mut self) {
        if !self.field_buffer.is_empty() {
            let name = match str::from_utf8(&self.field_buffer) {
                Ok(s) => Some(String::from(s)),
                _ => {
                    // invalid UTF-8 sequence in name
                    None
                }
            };

            let value = match str::from_utf8(&self.value_buffer) {
                Ok(s) => Some(String::from(s)),
                _ => {
                    // invalid UTF-8 sequence in value
                    None
                }
            };

            if name.is_some() && value.is_some() {
                self.parameters.push(name.unwrap(), value.unwrap());
            }
        }

        self.field_buffer.clear();
        self.value_buffer.clear();
    }

    /// Indicates that `parameter` exists within the collection of parameters.
    pub fn has_parameter<T: AsRef<str>>(&self, parameter: T) -> bool {
        self.parameters.has_parameter(parameter)
    }

    /// Indicates that parsing has finished.
    pub fn is_finished(&self) -> bool {
        self.finished
    }

    /// Retrieve `parameter` from the collection of parameters.
    pub fn parameter<T: AsRef<str>>(&self, parameter: T) -> Option<&ParameterValue> {
        self.parameters.parameter(parameter)
    }

    /// Retrieve the parameters.
    pub fn parameters(&self) -> &ParameterMap {
        &self.parameters
    }

    /// Reset the handler to its original state.
    pub fn reset(&mut self) {
        self.finished = false;
        self.toggle   = false;

        self.field_buffer.clear();
        self.parameters.clear();
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
