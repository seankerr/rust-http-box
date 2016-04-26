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

#![crate_name = "http_box"]

#![cfg_attr(test, feature(plugin))]
#![cfg_attr(test, plugin(clippy))]

#[macro_use]
extern crate bitflags;

#[macro_use]
pub mod byte;
pub mod http1;
pub mod url;

#[cfg(test)]
mod test;

// -------------------------------------------------------------------------------------------------

/// Crate major version.
pub const VERSION_MAJOR: &'static str = env!("CARGO_PKG_VERSION_MAJOR");

/// Crate minor version.
pub const VERSION_MINOR: &'static str = env!("CARGO_PKG_VERSION_MINOR");

/// Crate patch version.
pub const VERSION_PATCH: &'static str = env!("CARGO_PKG_VERSION_PATCH");

// -------------------------------------------------------------------------------------------------

/// Success response types.
#[derive(Clone,Copy,PartialEq)]
pub enum Success {
    /// Callback returned false.
    Callback(usize),

    /// Additional data expected.
    Eof(usize),

    /// Finished successfully.
    Finished(usize)
}
