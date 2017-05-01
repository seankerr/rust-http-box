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

#![crate_name = "http_box"]

#[macro_use]
extern crate byte_slice;

#[macro_use]
pub mod fsm;

#[macro_use]
mod collect;

pub mod byte;
pub mod http1;
// pub mod http2;
pub mod util;

#[cfg(test)]
mod test;

/// Crate major version.
pub const VERSION_MAJOR: &'static str = env!("CARGO_PKG_VERSION_MAJOR");

/// Crate minor version.
pub const VERSION_MINOR: &'static str = env!("CARGO_PKG_VERSION_MINOR");

/// Crate patch version.
pub const VERSION_PATCH: &'static str = env!("CARGO_PKG_VERSION_PATCH");
