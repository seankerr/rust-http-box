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

//! HTTP 2.x callback trait.

/// Type that handles HTTP/2.x parser events.
#[allow(unused_variables)]
pub trait HttpHandler {
    /// Callback that is executed when a data frame has been located.
    ///
    /// *Note:* This may be executed multiple times in order to supply the entire segment.
    ///
    /// **Arguments:**
    ///
    /// **`data`**
    ///
    /// The data.
    ///
    /// **`finished`**
    ///
    /// Indicates this is the last chunk of the data.
    ///
    /// **Returns:**
    ///
    /// `true` when parsing should continue, `false` to exit the parser function prematurely with
    /// [`Success::Callback`](../fsm/enum.Success.html#variant.Callback).
    fn on_data(&mut self, data: &[u8], finished: bool) -> bool {
        true
    }

    /// Callback that is executed when a new frame format has been located.
    ///
    /// **Arguments:**
    ///
    /// **`payload_length`**
    ///
    /// The payload length.
    ///
    /// **`frame_type`**
    ///
    /// The type.
    ///
    /// **`flags`**
    ///
    /// The flags.
    ///
    /// **`stream_id`**
    ///
    /// The stream identifier.
    ///
    /// **Returns:**
    ///
    /// `true` when parsing should continue, `false` to exit the parser function prematurely with
    /// [`Success::Callback`](../fsm/enum.Success.html#variant.Callback).
    fn on_frame_format(&mut self, payload_length: u32, frame_type: u8, flags: u8, stream_id: u32)
    -> bool {
        true
    }

    /// Callback that is executed when a go away frame has been located.
    ///
    /// **Arguments:**
    ///
    /// **`stream_id`**
    ///
    /// The stream identifier.
    ///
    /// **`error_code`**
    ///
    /// The error code.
    ///
    /// **Returns:**
    ///
    /// `true` when parsing should continue, `false` to exit the parser function prematurely with
    /// [`Success::Callback`](../fsm/enum.Success.html#variant.Callback).
    fn on_go_away(&mut self, stream_id: u32, error_code: u32) -> bool {
        true
    }

    /// Callback that is executed when go away frame debug data has been located.
    ///
    /// *Note:* This may be executed multiple times in order to supply the entire segment.
    ///
    /// **Arguments:**
    ///
    /// **`data`**
    ///
    /// The data.
    ///
    /// **`finished`**
    ///
    /// Indicates this is the last chunk of the data.
    ///
    /// **Returns:**
    ///
    /// `true` when parsing should continue, `false` to exit the parser function prematurely with
    /// [`Success::Callback`](../fsm/enum.Success.html#variant.Callback).
    fn on_go_away_debug_data(&mut self, data: &[u8], finished: bool) -> bool {
        true
    }

    /// Callback that is executed when a headers frame has been located.
    ///
    /// **Arguments:**
    ///
    /// **`exclusive`**
    ///
    /// Indicates the stream identifier is exclusive.
    ///
    /// **`stream_id`**
    ///
    /// The stream identifier.
    ///
    /// **`weight`**
    ///
    /// The weight.
    ///
    /// **Returns:**
    ///
    /// `true` when parsing should continue, `false` to exit the parser function prematurely with
    /// [`Success::Callback`](../fsm/enum.Success.html#variant.Callback).
    fn on_headers(&mut self, exclusive: bool, stream_id: u32, weight: u8) -> bool {
        true
    }

    /// Callback that is executed when a headers frame fragment has been located.
    ///
    /// *Note:* This may be executed multiple times in order to supply the entire segment.
    ///
    /// **Arguments:**
    ///
    /// **`fragment`**
    ///
    /// The fragment.
    ///
    /// **`finished`**
    ///
    /// Indicates this is the last chunk of the fragment.
    ///
    /// **Returns:**
    ///
    /// `true` when parsing should continue, `false` to exit the parser function prematurely with
    /// [`Success::Callback`](../fsm/enum.Success.html#variant.Callback).
    fn on_headers_fragment(&mut self, fragment: &[u8], finished: bool) -> bool {
        true
    }

    /// Callback that is executed when a ping frame has been located.
    ///
    /// *Note:* This may be executed multiple times in order to supply the entire segment.
    ///
    /// **Arguments:**
    ///
    /// **`data`**
    ///
    /// The data.
    ///
    /// **`finished`**
    ///
    /// Indicates this is the last chunk of the data.
    ///
    /// **Returns:**
    ///
    /// `true` when parsing should continue, `false` to exit the parser function prematurely with
    /// [`Success::Callback`](../fsm/enum.Success.html#variant.Callback).
    fn on_ping(&mut self, data: &[u8], finished: bool) -> bool {
        true
    }

    /// Callback that is executed when a priority frame has been located.
    ///
    /// **Arguments:**
    ///
    /// **`exclusive`**
    ///
    /// Indicates the stream identifier is exclusive.
    ///
    /// **`stream_id`**
    ///
    /// The stream identifier.
    ///
    /// **`weight`**
    ///
    /// The weight.
    ///
    /// **Returns:**
    ///
    /// `true` when parsing should continue, `false` to exit the parser function prematurely with
    /// [`Success::Callback`](../fsm/enum.Success.html#variant.Callback).
    fn on_priority(&mut self, exclusive: bool, stream_id: u32, weight: u8) -> bool {
        true
    }

    /// Callback that is executed when a push promise frame has been located.
    ///
    /// **Arguments:**
    ///
    /// **`stream_id`**
    ///
    /// The stream identifier.
    ///
    /// **Returns:**
    ///
    /// `true` when parsing should continue, `false` to exit the parser function prematurely with
    /// [`Success::Callback`](../fsm/enum.Success.html#variant.Callback).
    fn on_push_promise(&mut self, stream_id: u32) -> bool {
        true
    }

    /// Callback that is executed when a rst stream frame has been located.
    ///
    /// **Arguments:**
    ///
    /// **`error_code`**
    ///
    /// The error code.
    ///
    /// **Returns:**
    ///
    /// `true` when parsing should continue, `false` to exit the parser function prematurely with
    /// [`Success::Callback`](../fsm/enum.Success.html#variant.Callback).
    fn on_rst_stream(&mut self, error_code: u32) -> bool {
        true
    }

    /// Callback that is executed when a settings frame has been located.
    ///
    /// **Arguments:**
    ///
    /// **`id`**
    ///
    /// The identifier.
    ///
    /// **`value`**
    ///
    /// The value.
    ///
    /// **Returns:**
    ///
    /// `true` when parsing should continue, `false` to exit the parser function prematurely with
    /// [`Success::Callback`](../fsm/enum.Success.html#variant.Callback).
    fn on_settings(&mut self, id: u16, value: u32) -> bool {
        true
    }

    /// Callback that is executed when an unsupported frame has been located.
    ///
    /// *Note:* This may be executed multiple times in order to supply the entire segment.
    ///
    /// **Arguments:**
    ///
    /// **`data`**
    ///
    /// The data.
    ///
    /// **`finished`**
    ///
    /// Indicates this is the last chunk of the data.
    ///
    /// **Returns:**
    ///
    /// `true` when parsing should continue, `false` to exit the parser function prematurely with
    /// [`Success::Callback`](../fsm/enum.Success.html#variant.Callback).
    fn on_unsupported(&mut self, data: &[u8], finished: bool) -> bool {
        true
    }

    /// Callback that is executed when a window update frame has been located.
    ///
    /// **Arguments:**
    ///
    /// **`size_increment`**
    ///
    /// The size increment.
    ///
    /// **Returns:**
    ///
    /// `true` when parsing should continue, `false` to exit the parser function prematurely with
    /// [`Success::Callback`](../fsm/enum.Success.html#variant.Callback).
    fn on_window_update(&mut self, size_increment: u32) -> bool {
        true
    }
}
