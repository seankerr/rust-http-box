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

macro_rules! setup {
    ($parser:expr, $handler:expr) => ({
        $handler.set_transfer_encoding(TransferEncoding::Chunked);

        setup(&mut $parser, &mut $handler, b"GET / HTTP/1.1\r\n\r\nF;extension1=",
              State::ChunkExtensionValue);
    });
}

#[test]
fn byte_check_unquoted() {
    // invalid bytes
    loop_unsafe(b"\r;\"\\", |byte| {
        let mut h = DebugHandler::new();
        let mut p = Parser::new_request();

        setup!(p, h);

        assert_eof(&mut p, &mut h, &[b'"'], State::ChunkExtensionQuotedValue, 1);

        if let ParserError::ChunkExtensionValue(x) = assert_error(&mut p, &mut h,
                                                                  &[byte]).unwrap() {
            assert_eq!(x, byte);
        } else {
            panic!();
        }
    });

    // valid bytes
    loop_safe(b"\"\\", |byte| {
        let mut h = DebugHandler::new();
        let mut p = Parser::new_request();

        setup!(p, h);

        assert_eof(&mut p, &mut h, &[b'"'], State::ChunkExtensionQuotedValue, 1);
        assert_eof(&mut p, &mut h, &[byte], State::ChunkExtensionQuotedValue, 1);
    });
}

#[test]
fn basic() {
    let mut h = DebugHandler::new();
    let mut p = Parser::new_request();

    setup!(p, h);

    assert_eof(&mut p, &mut h, b"\"valid-value\"", State::ChunkExtensionSemiColon, 13);
    assert_eq!(h.chunk_extension_value, b"valid-value");
}

#[test]
fn escaped() {
    let mut h = DebugHandler::new();
    let mut p = Parser::new_request();

    setup!(p, h);

    assert_eof(&mut p, &mut h, b"\"valid \\\"value\\\" here\"\r", State::ChunkSizeNewline, 23);
    assert_eq!(h.chunk_extension_value, b"valid \"value\" here");
}

#[test]
fn maximum_length() {
    let mut h = DebugHandler::new();
    let mut p = Parser::new_request();

    setup!(p, h);

    assert_eof(&mut p, &mut h, &[b'a'; 243], State::ChunkExtensionValue, 243);
    vec_eq(&h.chunk_extension_value, &[b'a'; 243]);

    if let ParserError::MaxChunkExtensionLength = assert_error(&mut p, &mut h, &[b'a']).unwrap() {
    } else {
        panic!();
    }
}

#[test]
fn repeat() {
    let mut h = DebugHandler::new();
    let mut p = Parser::new_request();

    setup!(p, h);

    assert_eof(&mut p, &mut h, b"valid-value1;extension2=valid-value2;", State::ChunkExtensionName, 37);
    assert_eq!(h.chunk_extension_name, b"extension1extension2");
    assert_eq!(h.chunk_extension_value, b"valid-value1valid-value2");
}
