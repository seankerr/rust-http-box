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

use http1::*;
use test::*;
use test::http1::*;

macro_rules! setup {
    () => ({
        let mut parser = Parser::new_chunked(DebugHandler::new());

        assert_eos!(parser,
                    b"F;",
                    StripChunkExtensionName);

        parser
    });
}

#[test]
fn byte_check() {
    // invalid bytes
    loop_non_tokens(b"\r\t=; ", |byte| {
        let mut p = setup!();

        assert_error_byte!(p,
                           &[b'a', byte],
                           ChunkExtensionName,
                           byte);
    });

    // valid bytes
    loop_tokens(b"", |byte| {
        let mut p = setup!();

        assert_eos!(p,
                    &[byte],
                    LowerChunkExtensionName);
    });
}

#[test]
fn callback_exit() {
    struct CallbackHandler;

    impl HttpHandler for CallbackHandler {
        fn on_chunk_extension_name(&mut self, _name: &[u8]) -> bool {
            false
        }
    }

    let mut p = Parser::new_chunked(CallbackHandler);

    assert_eos!(p,
                b"F;",
                StripChunkExtensionName);

    // because chunk extension name is processed by 2 states, the callback exit will first
    // happen on the first byte
    assert_callback!(p,
                     b"ChunkExtension=",
                     LowerChunkExtensionName,
                     1);
}

#[test]
fn normalize() {
    let mut p = setup!();

    assert_eos!(p,
                b"CHANGE----LOWER",
                LowerChunkExtensionName);

    assert_eq!(p.handler().chunk_extension_name,
               b"change----lower");
}

#[test]
fn no_value() {
    let mut p = setup!();

    assert_eos!(p,
                b"valid-extension;",
                StripChunkExtensionName);

    assert_eq!(p.handler().chunk_extension_name,
               b"valid-extension");

    assert_eq!(p.handler().chunk_extension_value,
               b"");
}

#[test]
fn valid() {
    let mut p = setup!();

    assert_eos!(p,
                b"valid-extension=",
                StripChunkExtensionValue);

    assert_eq!(p.handler().chunk_extension_name,
               b"valid-extension");
}
