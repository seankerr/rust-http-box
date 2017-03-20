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
// | Author: Sean Kerr <sean@metatomic.io>                                                         |
// +-----------------------------------------------------------------------------------------------+

//! HTTP/2.x parser, errors, traits, and types.

mod error_code;
mod flags;
mod frame_format;
mod frame_type;
mod http_handler;
mod parser;
mod parser_state;
mod setting;

pub use http2::error_code::ErrorCode;
pub use http2::flags::Flags;
pub use http2::frame_format::FrameFormat;
pub use http2::frame_type::FrameType;
pub use http2::http_handler::HttpHandler;
pub use http2::parser::Parser;
pub use http2::parser_state::ParserState;
pub use http2::setting::Setting;
