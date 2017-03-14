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
use test::http1::*;

macro_rules! setup {
    () => ({
        let mut handler = DebugHandler::new();
        let mut parser  = Parser::new();

        parser.init_multipart();
        parser.set_boundary(b"XXDebugBoundaryXX");

        (parser, handler)
    });
}

#[test]
fn first_boundary_hyphen1_error () {
    let (mut p, mut h) = setup!();

    assert_error_byte!(
        p,
        h,
        b"@",
        MultipartBoundary,
        b'@'
    );
}

#[test]
fn first_boundary_hyphen2_error () {
    let (mut p, mut h) = setup!();

    assert_error_byte!(
        p,
        h,
        b"-@",
        MultipartBoundary,
        b'@'
    );
}

#[test]
fn first_boundary_match () {
    let (mut p, mut h) = setup!();

    assert_eos!(
        p,
        h,
        b"",
        MultipartHyphen1
    );

    assert_eos!(
        p,
        h,
        b"-",
        MultipartHyphen2
    );

    assert_eos!(
        p,
        h,
        b"-",
        MultipartBoundary
    );

    assert_eos!(
        p,
        h,
        b"XXDebugBoundary",
        MultipartBoundary
    );

    assert_eos!(
        p,
        h,
        b"XX",
        MultipartBoundaryCr
    );

    assert_eos!(
        p,
        h,
        b"\r",
        PreHeadersLf1
    );

    assert_eos!(
        p,
        h,
        b"\n",
        PreHeadersCr2
    );

    assert_eos!(
        p,
        h,
        b"\r",
        HeaderLf2
    );

    assert_eos!(
        p,
        h,
        b"\n",
        MultipartDataByByte
    );
}

#[test]
fn first_boundary_no_match () {
    let (mut p, mut h) = setup!();

    assert_eos!(
        p,
        h,
        b"--XXDebugBoundary",
        MultipartBoundary
    );

    assert_error_byte!(
        p,
        h,
        b"Q",
        MultipartBoundary,
        b'Q'
    );
}

#[test]
fn second_boundary_match () {
    let (mut p, mut h) = setup!();

    assert_eos!(
        p,
        h,
        b"--XXDebugBoundaryXX\r\n\
         \r\n\
         DATA1\r\n\
         --XXDebugBoundaryXX\r\n",
        PreHeadersCr2
    );

    assert_eq!(
        h.multipart_data,
        b"DATA1"
    );
}

#[test]
fn second_false_boundary () {
    let (mut p, mut h) = setup!();

    assert_eos!(
        p,
        h,
        b"--XXDebugBoundaryXX\r\n\
         \r\n\
         DATA1\r\n\
         --XXDebugBoundaryQ",
        MultipartDataByByte
    );

    assert_eq!(
        h.multipart_data,
        b"DATA1\r\n--XXDebugBoundaryQ"
    );
}

#[test]
fn second_false_third_boundary_match () {
    let (mut p, mut h) = setup!();

    assert_eos!(
        p,
        h,
        b"--XXDebugBoundaryXX\r\n\
         \r\n\
         DATA1\r\n\
         --XXDebugBoundaryQ\r\n\
         --XXDebugBoundaryXX\r\n\
         \r\n\
         ABCD",
        MultipartDataByByte
    );

    assert_eq!(
        h.multipart_data,
        b"DATA1\r\n--XXDebugBoundaryQABCD"
    );
}
