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

use handler::*;
use http1::*;
use test::*;
use test::http1::*;

macro_rules! setup {
    ($parser:expr, $handler:expr) => ({
        chunked_setup(&mut $parser, &mut $handler, b"F;", State::ChunkExtensionName);
    });
}

#[test]
fn byte_check() {
    // invalid bytes
    loop_non_tokens(b"\r=\"", |byte| {
        let mut h = DebugHandler::new();
        let mut p = Parser::new();

        setup!(p, h);

        if let ParserError::ChunkExtensionName(x) = chunked_assert_error(&mut p, &mut h,
                                                                         &[byte]).unwrap() {
            assert_eq!(x, byte);
        } else {
            panic!();
        }
    });

    // valid bytes
    loop_tokens(b"", |byte| {
        let mut h = DebugHandler::new();
        let mut p = Parser::new();

        setup!(p, h);

        chunked_assert_eos(&mut p, &mut h, &[byte], State::ChunkExtensionName, 1);
    });
}

#[test]
fn callback_exit() {
    struct X;

    impl HttpHandler for X {
        fn on_chunk_extension_name(&mut self, _name: &[u8]) -> bool {
            false
        }
    }

    let mut h = X{};
    let mut p = Parser::new();

    setup!(p, h);

    chunked_assert_callback(&mut p, &mut h, b"ChunkExtension=", State::ChunkExtensionValue, 15);
}

#[test]
fn valid() {
    let mut h = DebugHandler::new();
    let mut p = Parser::new();

    setup!(p, h);

    chunked_assert_eos(&mut p, &mut h, b"valid-extension=", State::ChunkExtensionValue, 16);
    assert_eq!(h.chunk_extension_name, b"valid-extension");
}
