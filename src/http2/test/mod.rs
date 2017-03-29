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

use http2::*;

macro_rules! pack_bytes {
    ($buffer:expr, $bytes:expr) => ({
        $buffer.extend_from_slice($bytes);
    });
}

macro_rules! pack_u8 {
    ($buffer:expr, $byte:expr) => ({
        $buffer.push($byte);
    });
}

macro_rules! pack_u16 {
    ($buffer:expr, $bytes:expr) => ({
        $buffer.push(($bytes as u16 >> 8) as u8);
        $buffer.push(($bytes as u16 & 0xFF) as u8);
    });
}

macro_rules! pack_u32 {
    ($buffer:expr, $bytes:expr) => ({
        $buffer.push(($bytes as u32 >> 24) as u8);
        $buffer.push((($bytes as u32 >> 16) & 0xFF) as u8);
        $buffer.push((($bytes as u32 >> 8) & 0xFF) as u8);
        $buffer.push(($bytes as u32 & 0xFF) as u8);
    });
}

// -------------------------------------------------------------------------------------------------

/// `DebugHandler` works with all `http2::Parser` parsing methods.
///
/// When in use, all parsed bytes will be printed, along with the callback name and length
/// of parsed data.
///
/// If you're debugging large sets of data, it's a good idea to pass fairly small chunks
/// of stream data at a time, about *4096* bytes or so. And in between parser function calls, if
/// you don't need to retain the data, execute
/// [`reset()`](struct.DebugHandler.html#method.reset) so that vectors
/// collecting the data don't consume too much memory.
#[derive(Default)]
pub struct DebugHandler {
    /// Data frame data.
    pub data_data: Vec<u8>,

    /// Data frame finished.
    pub data_data_finished: bool,

    /// Frame flags.
    pub frame_flags: u8,

    /// Frame payload length.
    pub frame_payload_length: u32,

    /// Frame stream id.
    pub frame_stream_id: u32,

    /// Frame type.
    pub frame_type: u8,

    /// Go away debug data.
    pub go_away_debug_data: Vec<u8>,

    /// Go away debug data finished.
    pub go_away_debug_data_finished: bool,

    /// Go away error code.
    pub go_away_error_code: u32,

    /// Go away stream id.
    pub go_away_stream_id: u32,

    /// Headers data.
    pub headers_data: Vec<u8>,

    /// Headers data finished.
    pub headers_data_finished: bool,

    /// Headers exclusive.
    pub headers_exclusive: bool,

    /// Headers stream id.
    pub headers_stream_id: u32,

    /// Headers weight.
    pub headers_weight: u8,

    /// Ping data.
    pub ping_data: Vec<u8>,

    /// Ping data finished.
    pub ping_data_finished: bool,

    /// Priority exclusive.
    pub priority_exclusive: bool,

    /// Priority stream id.
    pub priority_stream_id: u32,

    /// Priority weight.
    pub priority_weight: u8,

    /// Push promise stream id.
    pub push_promise_stream_id: u32,

    /// Rst stream error code.
    pub rst_stream_error_code: u32,

    /// Settings id.
    pub settings_id: u16,

    /// Settings value.
    pub settings_value: u32,

    /// Unsupported data.
    pub unsupported_data: Vec<u8>,

    /// Unsupported data finished.
    pub unsupported_data_finished: bool,

    /// Window update size increment.
    pub window_update_size_increment: u32
}

impl DebugHandler {
    /// Create a new `DebugHandler`.
    pub fn new() -> DebugHandler {
        DebugHandler{
            data_data:                    Vec::new(),
            data_data_finished:           false,
            frame_flags:                  0,
            frame_payload_length:         0,
            frame_stream_id:              0,
            frame_type:                   0,
            go_away_debug_data:           Vec::new(),
            go_away_debug_data_finished:  false,
            go_away_error_code:           0,
            go_away_stream_id:            0,
            headers_data:                 Vec::new(),
            headers_data_finished:        false,
            headers_exclusive:            false,
            headers_stream_id:            0,
            headers_weight:               0,
            ping_data:                    Vec::new(),
            ping_data_finished:           false,
            priority_exclusive:           false,
            priority_stream_id:           0,
            priority_weight:              0,
            push_promise_stream_id:       0,
            rst_stream_error_code:        0,
            settings_id:                  0,
            settings_value:               0,
            unsupported_data:             Vec::new(),
            unsupported_data_finished:    false,
            window_update_size_increment: 0
        }
    }
}

impl HttpHandler for DebugHandler {
    fn on_data(&mut self, data: &[u8], finished: bool) -> bool {
        println!(
            "on_data [{}, {}]: {:?}",
            data.len(),
            finished,
            data
        );

        self.data_data.extend_from_slice(data);
        self.data_data_finished = finished;
        true
    }

    fn on_frame_format(&mut self, payload_length: u32, frame_type: u8, flags: u8, stream_id: u32)
    -> bool {
        println!(
            "on_frame_format: payload_length={}, frame_type={}, flags={}, stream_id={}",
            payload_length,
            FrameType::from_u8(frame_type),
            Flags::from_u8(flags),
            stream_id
        );

        self.frame_flags          = flags;
        self.frame_payload_length = payload_length;
        self.frame_stream_id      = stream_id;
        self.frame_type           = frame_type;
        true
    }

    fn on_go_away(&mut self, stream_id: u32, error_code: u32) -> bool {
        println!(
            "on_go_away: stream_id={}, error_code={}",
            stream_id,
            error_code
        );

        self.go_away_error_code = error_code;
        self.go_away_stream_id  = stream_id;
        true
    }

    fn on_go_away_debug_data(&mut self, data: &[u8], finished: bool) -> bool {
        println!(
            "on_go_away_debug_data [{}, {}]: {:?}",
            data.len(),
            finished,
            data
        );

        self.go_away_debug_data.extend_from_slice(data);
        self.go_away_debug_data_finished = finished;
        true
    }

    fn on_headers(&mut self, exclusive: bool, stream_id: u32, weight: u8) -> bool {
        println!(
            "on_headers: exclusive={}, stream_id={}, weight={}",
            exclusive,
            stream_id,
            weight
        );

        self.headers_exclusive = exclusive;
        self.headers_stream_id = stream_id;
        self.headers_weight    = weight;
        true
    }

    fn on_headers_fragment(&mut self, fragment: &[u8], finished: bool) -> bool {
        println!(
            "on_headers_fragment [{}, {}]: {:?}",
            fragment.len(),
            finished,
            &fragment
        );

        self.headers_data.extend_from_slice(fragment);
        self.headers_data_finished = finished;
        true
    }

    fn on_ping(&mut self, data: &[u8], finished: bool) -> bool {
        println!(
            "on_ping [{}, {}]: {:?}",
            data.len(),
            finished,
            &data
        );

        self.ping_data.extend_from_slice(data);
        self.ping_data_finished = finished;
        true
    }

    fn on_priority(&mut self, exclusive: bool, stream_id: u32, weight: u8) -> bool {
        println!(
            "on_priority: exclusive={}, stream_id={}, weight={}",
            exclusive,
            stream_id,
            weight
        );

        self.priority_exclusive = exclusive;
        self.priority_stream_id = stream_id;
        self.priority_weight    = weight;
        true
    }

    fn on_push_promise(&mut self, stream_id: u32) -> bool {
        println!(
            "on_push_promise: stream_id={}",
            stream_id
        );

        self.push_promise_stream_id = stream_id;
        true
    }

    fn on_rst_stream(&mut self, error_code: u32) -> bool {
        println!(
            "on_rst_stream: error_code={}",
            error_code
        );

        self.rst_stream_error_code = error_code;
        true
    }

    fn on_settings(&mut self, id: u16, value: u32) -> bool {
        println!(
            "on_settings: id={}, value={}",
            id,
            value
        );

        self.settings_id    = id;
        self.settings_value = value;
        true
    }

    fn on_unsupported(&mut self, data: &[u8], finished: bool) -> bool {
        println!(
            "on_unsupported [{}, {}]: {:?}",
            data.len(),
            finished,
            &data
        );

        self.unsupported_data.extend_from_slice(data);
        self.unsupported_data_finished = finished;
        true
    }

    fn on_window_update(&mut self, size_increment: u32) -> bool {
        println!(
            "on_window_update: size_increment={}",
            size_increment
        );

        self.window_update_size_increment = size_increment;
        true
    }
}

// -------------------------------------------------------------------------------------------------

// test mods
mod continuation;
mod data;
mod frame_format;
mod go_away;
mod headers;
mod ping;
mod priority;
mod push_promise;
mod rst_stream;
mod settings;
mod window_update;
