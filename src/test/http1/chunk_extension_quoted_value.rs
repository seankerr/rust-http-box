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

use http1::*;
use test::*;
use test::http1::*;

macro_rules! setup {
    () => ({
        let mut handler = DebugHandler::new();
        let mut parser  = Parser::new();

        parser.init_chunked();

        assert_eos!(
            parser,
            handler,
            b"F;extension1=",
            StripChunkExtensionValue
        );

        (parser, handler)
    });
}

#[test]
fn basic() {
    let (mut p, mut h) = setup!();

    assert_eos!(
        p,
        h,
        b"\"valid-value\"",
        ChunkExtensionQuotedValueFinished
    );

    assert_eq!(
        h.chunk_extension_value,
        b"valid-value"
    );
}

#[test]
fn byte_check() {
    // invalid bytes
    loop_non_quoted(
        b"\r;\"\\",
        |byte| {
            let (mut p, mut h) = setup!();

            assert_eos!(
                p,
                h,
                &[b'"'],
                ChunkExtensionQuotedValue
            );

            assert_error_byte!(
                p,
                h,
                &[byte],
                ChunkExtensionValue,
                byte
            );
        }
    );

    // valid bytes
    loop_quoted(
        b"\"\\",
        |byte| {
            let (mut p, mut h) = setup!();

            assert_eos!(
                p,
                h,
                &[b'"'],
                ChunkExtensionQuotedValue
            );

            assert_eos!(
                p,
                h,
                &[byte],
                ChunkExtensionQuotedValue
            );
        }
    );
}

#[test]
fn callback_exit() {
    struct CallbackHandler;

    impl HttpHandler for CallbackHandler {
        fn on_chunk_extension_value(&mut self, _value: &[u8]) -> bool {
            false
        }
    }

    let mut h = CallbackHandler;
    let mut p = Parser::new();

    p.init_chunked();

    assert_eos!(
        p,
        h,
        b"F;extension1=",
        StripChunkExtensionValue
    );

    assert_callback!(
        p,
        h,
        b"\"ExtensionValue\"",
        ChunkExtensionQuotedValueFinished
    );
}

#[test]
fn escaped() {
    let (mut p, mut h) = setup!();

    assert_eos!(
        p,
        h,
        b"\"valid \\\"value\\\" here\"\r",
        ChunkLengthLf
    );

    assert_eq!(
        h.chunk_extension_value,
        b"valid \"value\" here"
    );
}

#[test]
fn repeat() {
    let (mut p, mut h) = setup!();

    assert_eos!(
        p,
        h,
        b"valid-value1;extension2=valid-value2;",
        StripChunkExtensionName
    );

    assert_eq!(
        h.chunk_extension_name,
        b"extension1extension2"
    );

    assert_eq!(
        h.chunk_extension_value,
        b"valid-value1valid-value2"
    );
}
