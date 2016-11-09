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
use test::http1::*;

macro_rules! setup {
    ($parser:expr, $handler:expr) => ({
        $parser.init_multipart(b"XXDebugBoundaryXX");
    });
}

#[test]
fn first_boundary_hyphen1_error () {
    let mut h = DebugHandler::new();
    let mut p = Parser::new();

    setup!(p, h);

    assert_error_byte!(p, h,
                       b"@",
                       ParserError::MultipartBoundary,
                       b'@');
}

#[test]
fn first_boundary_hyphen2_error () {
    let mut h = DebugHandler::new();
    let mut p = Parser::new();

    setup!(p, h);

    assert_error_byte!(p, h,
                       b"-@",
                       ParserError::MultipartBoundary,
                       b'@');
}

#[test]
fn first_boundary_match () {
    let mut h = DebugHandler::new();
    let mut p = Parser::new();

    setup!(p, h);

    assert_eos!(p, h,
                b"",
                ParserState::MultipartHyphen1);

    assert_eos!(p, h,
                b"-",
                ParserState::MultipartHyphen2);

    assert_eos!(p, h,
                b"-",
                ParserState::MultipartBoundary);

    assert_eos!(p, h,
                b"XXDebugBoundary",
                ParserState::MultipartBoundary);

    assert_eos!(p, h,
                b"XX",
                ParserState::MultipartBoundaryCr);

    assert_eos!(p, h,
                b"\r",
                ParserState::PreHeadersLf1);

    assert_eos!(p, h,
                b"\n",
                ParserState::PreHeadersCr2);

    assert_eos!(p, h,
                b"\r",
                ParserState::HeaderLf2);

    assert_eos!(p, h,
                b"\n",
                ParserState::MultipartDataByByte);
}

#[test]
fn first_boundary_no_match () {
    let mut h = DebugHandler::new();
    let mut p = Parser::new();

    setup!(p, h);

    assert_eos!(p, h,
                b"--XXDebugBoundary",
                ParserState::MultipartBoundary);

    assert_error_byte!(p, h,
                       b"Q",
                       ParserError::MultipartBoundary,
                       b'Q');
}

#[test]
fn second_boundary_match () {
    let mut h = DebugHandler::new();
    let mut p = Parser::new();

    setup!(p, h);

    assert_eos!(p, h,
                b"--XXDebugBoundaryXX\r\n\
                  \r\n\
                  DATA1\r\n\
                  --XXDebugBoundaryXX\r\n",
                ParserState::PreHeadersCr2);

    assert_eq!(h.multipart_data, b"DATA1");
}

#[test]
fn second_false_boundary () {
    let mut h = DebugHandler::new();
    let mut p = Parser::new();

    setup!(p, h);

    assert_eos!(p, h,
                b"--XXDebugBoundaryXX\r\n\
                  \r\n\
                  DATA1\r\n\
                  --XXDebugBoundaryQ",
                ParserState::MultipartDataByByte);

    assert_eq!(h.multipart_data, b"DATA1\r\n--XXDebugBoundaryQ");
}

#[test]
fn second_false_third_boundary_match () {
    let mut h = DebugHandler::new();
    let mut p = Parser::new();

    setup!(p, h);

    assert_eos!(p, h,
                b"--XXDebugBoundaryXX\r\n\
                  \r\n\
                  DATA1\r\n\
                  --XXDebugBoundaryQ\r\n\
                  --XXDebugBoundaryXX\r\n\
                  \r\n\
                  ABCD",
                ParserState::MultipartDataByByte);

    assert_eq!(h.multipart_data, b"DATA1\r\n--XXDebugBoundaryQABCD");
}
