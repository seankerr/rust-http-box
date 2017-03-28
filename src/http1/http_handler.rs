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

/// Type that handles HTTP/1.x parser events.
#[allow(unused_variables)]
pub trait HttpHandler {
    /// Retrieve the content length.
    ///
    /// **Called When::**
    ///
    /// Within multipart parsing, after each boundary's head data has been parsed.
    fn content_length(&mut self) -> Option<usize> {
        None
    }

    /// Callback that is executed when body parsing has completed successfully.
    ///
    /// **Returns:**
    ///
    /// `true` when parsing should continue, `false` to exit the parser function prematurely with
    /// [`Success::Callback`](../fsm/enum.Success.html#variant.Callback).
    fn on_body_finished(&mut self) -> bool {
        true
    }

    /// Callback that is executed when a new chunk section has been located. This is executed
    /// prior to the length, extensions, and data.
    ///
    /// **Returns:**
    ///
    /// `true` when parsing should continue, `false` to exit the parser function prematurely with
    /// [`Success::Callback`](../fsm/enum.Success.html#variant.Callback).
    fn on_chunk_begin(&mut self) -> bool {
        true
    }

    /// Callback that is executed when chunk encoded data has been located.
    ///
    /// *Note:* This may be executed multiple times in order to supply the entire segment.
    ///
    /// **Returns:**
    ///
    /// `true` when parsing should continue, `false` to exit the parser function prematurely with
    /// [`Success::Callback`](../fsm/enum.Success.html#variant.Callback).
    fn on_chunk_data(&mut self, data: &[u8]) -> bool {
        true
    }

    /// Callback that is executed when parsing an individual chunk extension name/value pair has
    /// completed successfully.
    ///
    /// **Returns:**
    ///
    /// `true` when parsing should continue, `false` to exit the parser function prematurely with
    /// [`Success::Callback`](../fsm/enum.Success.html#variant.Callback).
    fn on_chunk_extension_finished(&mut self) -> bool {
        true
    }

    /// Callback that is executed when a chunk extension name has been located.
    ///
    /// *Note:* This may be executed multiple times in order to supply the entire segment.
    ///
    /// **Returns:**
    ///
    /// `true` when parsing should continue, `false` to exit the parser function prematurely with
    /// [`Success::Callback`](../fsm/enum.Success.html#variant.Callback).
    fn on_chunk_extension_name(&mut self, name: &[u8]) -> bool {
        true
    }

    /// Callback that is executed when a chunk extension value has been located.
    ///
    /// *Note:* This may be executed multiple times in order to supply the entire segment.
    ///
    /// **Returns:**
    ///
    /// `true` when parsing should continue, `false` to exit the parser function prematurely with
    /// [`Success::Callback`](../fsm/enum.Success.html#variant.Callback).
    fn on_chunk_extension_value(&mut self, value: &[u8]) -> bool {
        true
    }

    /// Callback that is executed when parsing all chunk extensions has completed successfully.
    ///
    /// **Returns:**
    ///
    /// `true` when parsing should continue, `false` to exit the parser function prematurely with
    /// [`Success::Callback`](../fsm/enum.Success.html#variant.Callback).
    fn on_chunk_extensions_finished(&mut self) -> bool {
        true
    }

    /// Callback that is executed when a chunk length has been located.
    ///
    /// **Returns:**
    ///
    /// `true` when parsing should continue, `false` to exit the parser function prematurely with
    /// [`Success::Callback`](../fsm/enum.Success.html#variant.Callback).
    fn on_chunk_length(&mut self, size: usize) -> bool {
        true
    }

    /// Callback that is executed when a header name has been located.
    ///
    /// *Note:* This may be executed multiple times in order to supply the entire segment.
    ///
    /// **Returns:**
    ///
    /// `true` when parsing should continue, `false` to exit the parser function prematurely with
    /// [`Success::Callback`](../fsm/enum.Success.html#variant.Callback).
    ///
    /// **Called When:**
    ///
    /// During head parsing, multipart parsing of headers for each piece of data, or at the end of
    /// chunk encoded data when trailers are present.
    fn on_header_name(&mut self, name: &[u8]) -> bool {
        true
    }

    /// Callback that is executed when a header value has been located.
    ///
    /// *Note:* This may be executed multiple times in order to supply the entire segment.
    ///
    /// **Returns:**
    ///
    /// `true` when parsing should continue, `false` to exit the parser function prematurely with
    /// [`Success::Callback`](../fsm/enum.Success.html#variant.Callback).
    ///
    /// **Called When:**
    ///
    /// During head parsing, multipart parsing of headers for each piece of data, or at the end of
    /// chunk encoded data when trailers are present.
    fn on_header_value(&mut self, value: &[u8]) -> bool {
        true
    }

    /// Callback that is executed when header parsing has completed successfully.
    ///
    /// **Returns:**
    ///
    /// `true` when parsing should continue, `false` to exit the parser function prematurely with
    /// [`Success::Callback`](../fsm/enum.Success.html#variant.Callback).
    ///
    /// **Called When:**
    ///
    /// During head parsing, multipart parsing of headers for each piece of data, or at the end of
    /// chunk encoded data when trailers are present.
    fn on_headers_finished(&mut self) -> bool {
        true
    }

    /// Callback that is executed when parsing the initial request/response line has completed
    /// successfully.
    ///
    /// **Returns:**
    ///
    /// `true` when parsing should continue, `false` to exit the parser function prematurely with
    /// [`Success::Callback`](../fsm/enum.Success.html#variant.Callback).
    fn on_initial_finished(&mut self) -> bool {
        true
    }

    /// Callback that is executed when a request method has been located.
    ///
    /// *Note:* This may be executed multiple times in order to supply the entire segment.
    ///
    /// **Returns:**
    ///
    /// `true` when parsing should continue, `false` to exit the parser function prematurely with
    /// [`Success::Callback`](../fsm/enum.Success.html#variant.Callback).
    fn on_method(&mut self, method: &[u8]) -> bool {
        true
    }

    /// Callback that is executed when a new multipart section has been located. This is executed
    /// prior to any headers.
    ///
    /// **Returns:**
    ///
    /// `true` when parsing should continue, `false` to exit the parser function prematurely with
    /// [`Success::Callback`](../fsm/enum.Success.html#variant.Callback).
    fn on_multipart_begin(&mut self) -> bool {
        true
    }

    /// Callback that is executed when multipart data has been located.
    ///
    /// *Note:* This may be executed multiple times in order to supply the entire segment.
    ///
    /// **Returns:**
    ///
    /// `true` when parsing should continue, `false` to exit the parser function prematurely with
    /// [`Success::Callback`](../fsm/enum.Success.html#variant.Callback).
    fn on_multipart_data(&mut self, data: &[u8]) -> bool {
        true
    }

    /// Callback that is executed when a response status has been located.
    ///
    /// *Note:* This may be executed multiple times in order to supply the entire segment.
    ///
    /// **Returns:**
    ///
    /// `true` when parsing should continue, `false` to exit the parser function prematurely with
    /// [`Success::Callback`](../fsm/enum.Success.html#variant.Callback).
    fn on_status(&mut self, status: &[u8]) -> bool {
        true
    }

    /// Callback that is executed when a response status code has been located.
    ///
    /// **Returns:**
    ///
    /// `true` when parsing should continue, `false` to exit the parser function prematurely with
    /// [`Success::Callback`](../fsm/enum.Success.html#variant.Callback).
    fn on_status_code(&mut self, code: u16) -> bool {
        true
    }

    /// Callback that is executed when a request URL has been located.
    ///
    /// *Note:* This may be executed multiple times in order to supply the entire segment.
    ///
    /// **Returns:**
    ///
    /// `true` when parsing should continue, `false` to exit the parser function prematurely with
    /// [`Success::Callback`](../fsm/enum.Success.html#variant.Callback).
    fn on_url(&mut self, url: &[u8]) -> bool {
        true
    }

    /// Callback that is executed when a URL encoded name has been located.
    ///
    /// *Note:* This may be executed multiple times in order to supply the entire segment.
    ///
    /// **Returns:**
    ///
    /// `true` when parsing should continue, `false` to exit the parser function prematurely with
    /// [`Success::Callback`](../fsm/enum.Success.html#variant.Callback).
    fn on_url_encoded_name(&mut self, name: &[u8]) -> bool {
        true
    }

    /// Callback that is executed when a URL encoded value has been located.
    ///
    /// *Note:* This may be executed multiple times in order to supply the entire segment.
    ///
    /// **Returns:**
    ///
    /// `true` when parsing should continue, `false` to exit the parser function prematurely with
    /// [`Success::Callback`](../fsm/enum.Success.html#variant.Callback).
    fn on_url_encoded_value(&mut self, value: &[u8]) -> bool {
        true
    }

    /// Callback that is executed when the HTTP version has been located during the initial request
    /// or response line.
    ///
    /// **Returns:**
    ///
    /// `true` when parsing should continue, `false` to exit the parser function prematurely with
    /// [`Success::Callback`](../fsm/enum.Success.html#variant.Callback).
    fn on_version(&mut self, major: u16, minor: u16) -> bool {
        true
    }
}
