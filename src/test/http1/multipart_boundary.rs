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
        let mut parser = Parser::new_multipart(DebugHandler::new());

        parser.set_boundary(b"XXDebugBoundaryXX");

        parser
    });
}

#[test]
fn first_boundary_hyphen1_error () {
    let mut p = setup!();

    assert_error_byte!(p,
                       b"@",
                       MultipartBoundary,
                       b'@');
}

#[test]
fn first_boundary_hyphen2_error () {
    let mut p = setup!();

    assert_error_byte!(p,
                       b"-@",
                       MultipartBoundary,
                       b'@');
}

#[test]
fn first_boundary_match () {
    let mut p = setup!();

    assert_eos!(p,
                b"",
                MultipartHyphen1);

    assert_eos!(p,
                b"-",
                MultipartHyphen2);

    assert_eos!(p,
                b"-",
                MultipartBoundary);

    assert_eos!(p,
                b"XXDebugBoundary",
                MultipartBoundary);

    assert_eos!(p,
                b"XX",
                MultipartBoundaryCr);

    assert_eos!(p,
                b"\r",
                PreHeadersLf1);

    assert_eos!(p,
                b"\n",
                PreHeadersCr2);

    assert_eos!(p,
                b"\r",
                HeaderLf2);

    assert_eos!(p,
                b"\n",
                MultipartDataByByte);
}

#[test]
fn first_boundary_no_match () {
    let mut p = setup!();

    assert_eos!(p,
                b"--XXDebugBoundary",
                MultipartBoundary);

    assert_error_byte!(p,
                       b"Q",
                       MultipartBoundary,
                       b'Q');
}

#[test]
fn second_boundary_match () {
    let mut p = setup!();

    assert_eos!(p,
                b"--XXDebugBoundaryXX\r\n\
                  \r\n\
                  DATA1\r\n\
                  --XXDebugBoundaryXX\r\n",
                PreHeadersCr2);

    assert_eq!(p.handler().multipart_data,
               b"DATA1");
}

#[test]
fn second_false_boundary () {
    let mut p = setup!();

    assert_eos!(p,
                b"--XXDebugBoundaryXX\r\n\
                  \r\n\
                  DATA1\r\n\
                  --XXDebugBoundaryQ",
                MultipartDataByByte);

    assert_eq!(p.handler().multipart_data,
               b"DATA1\r\n--XXDebugBoundaryQ");
}

#[test]
fn second_false_third_boundary_match () {
    let mut p = setup!();

    assert_eos!(p,
                b"--XXDebugBoundaryXX\r\n\
                  \r\n\
                  DATA1\r\n\
                  --XXDebugBoundaryQ\r\n\
                  --XXDebugBoundaryXX\r\n\
                  \r\n\
                  ABCD",
                MultipartDataByByte);

    assert_eq!(p.handler().multipart_data,
               b"DATA1\r\n--XXDebugBoundaryQABCD");
}
