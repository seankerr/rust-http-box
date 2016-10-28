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

#[test]
fn first_boundary_hyphen1_error () {
    let mut h = DebugHandler::new();
    let mut p = Parser::new();

    if let ParserError::MultipartBoundary(x) = multipart_assert_error(&mut p,
                                                                      &mut h,
                                                                      b"@").unwrap() {
        assert_eq!(x, b'@');
    } else {
        panic!();
    }
}

#[test]
fn first_boundary_hyphen2_error () {
    let mut h = DebugHandler::new();
    let mut p = Parser::new();

    if let ParserError::MultipartBoundary(x) = multipart_assert_error(&mut p,
                                                                      &mut h,
                                                                      b"-@").unwrap() {
        assert_eq!(x, b'@');
    } else {
        panic!();
    }
}

#[test]
fn first_boundary_match () {
    let mut h = DebugHandler::new();
    let mut p = Parser::new();

    multipart_assert_eos(&mut p, &mut h,
                         b"",
                         ParserState::MultipartHyphen1, 0);

    multipart_assert_eos(&mut p, &mut h,
                         b"-",
                         ParserState::MultipartHyphen2, 1);

    multipart_assert_eos(&mut p, &mut h,
                         b"-",
                         ParserState::MultipartBoundary, 1);

    multipart_assert_eos(&mut p, &mut h,
                         b"XXDebugBoundary",
                         ParserState::MultipartBoundary, 15);

    multipart_assert_eos(&mut p, &mut h,
                         b"XX",
                         ParserState::MultipartNewline1, 2);

    multipart_assert_eos(&mut p, &mut h,
                         b"\r",
                         ParserState::PreHeaders1, 1);

    multipart_assert_eos(&mut p, &mut h,
                         b"\n",
                         ParserState::PreHeaders2, 1);

    multipart_assert_eos(&mut p, &mut h,
                         b"\r",
                         ParserState::Newline4, 1);

    multipart_assert_eos(&mut p, &mut h,
                         b"\n",
                         ParserState::MultipartDataByByte, 1);
}

#[test]
fn first_boundary_no_match () {
    let mut h = DebugHandler::new();
    let mut p = Parser::new();

    multipart_assert_eos(&mut p, &mut h,
                         b"--XXDebugBoundary",
                         ParserState::MultipartBoundary, 17);

    if let ParserError::MultipartBoundary(x) = multipart_assert_error(&mut p,
                                                                      &mut h,
                                                                      b"Q").unwrap() {
        assert_eq!(x, b'Q');
    } else {
        panic!();
    }
}

#[test]
fn missing_boundary() {
    struct X;

    impl Http1Handler for X {
    }

    let mut h = X{};
    let mut p = Parser::new();

    if let ParserError::MultipartBoundaryExpected = multipart_assert_error(&mut p,
                                                                           &mut h,
                                                                           b"--X").unwrap() {
    } else {
        panic!();
    }
}

#[test]
fn second_boundary_match () {
    let mut h = DebugHandler::new();
    let mut p = Parser::new();

    multipart_assert_eos(&mut p, &mut h,
                         b"--XXDebugBoundaryXX\r\n\
                           \r\n\
                           DATA1\r\n\
                           --XXDebugBoundaryXX\r\n",
                         ParserState::PreHeaders2, 51);

    assert_eq!(h.multipart_data, b"DATA1");
}

#[test]
fn second_false_boundary () {
    let mut h = DebugHandler::new();
    let mut p = Parser::new();

    multipart_assert_eos(&mut p, &mut h,
                         b"--XXDebugBoundaryXX\r\n\
                           \r\n\
                           DATA1\r\n\
                           --XXDebugBoundaryQ",
                         ParserState::MultipartDataByByte, 48);

    assert_eq!(h.multipart_data, b"DATA1\r\n--XXDebugBoundaryQ");
}

#[test]
fn second_false_third_boundary_match () {
    let mut h = DebugHandler::new();
    let mut p = Parser::new();

    multipart_assert_eos(&mut p, &mut h,
                         b"--XXDebugBoundaryXX\r\n\
                           \r\n\
                           DATA1\r\n\
                           --XXDebugBoundaryQ\r\n\
                           --XXDebugBoundaryXX\r\n\
                           \r\n\
                           ABCD",
                         ParserState::MultipartDataByByte, 77);

    assert_eq!(h.multipart_data, b"DATA1\r\n--XXDebugBoundaryQABCD");
}
