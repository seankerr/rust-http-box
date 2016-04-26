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

use Success;
use http1::*;
use url::*;

struct H {}

impl HttpHandler for H {}

impl ParamHandler for H {}

#[test]
fn request_http_eof() {
    let mut h = H{};
    let mut p = Parser::new(StreamType::Request);

    assert!(match p.parse(&mut h, b"GET /path HTTP") {
        Ok(Success::Eof(_)) => true,
        _ => false
    });

    assert_eq!(p.get_state(), State::RequestHttp5);
}

#[test]
fn request_http_upper() {
    let mut h = H{};
    let mut p = Parser::new(StreamType::Request);

    assert!(match p.parse(&mut h, b"GET /path HTTP/") {
        Ok(Success::Eof(_)) => true,
        _ => false
    });

    assert_eq!(p.get_state(), State::RequestVersionMajor);
}

#[test]
fn request_http_lower() {
    let mut h = H{};
    let mut p = Parser::new(StreamType::Request);

    assert!(match p.parse(&mut h, b"GET /path http/") {
        Ok(Success::Eof(_)) => true,
        _ => false
    });

    assert_eq!(p.get_state(), State::RequestVersionMajor);
}

#[test]
fn request_http_multiple_streams() {
    let mut h = H{};
    let mut p = Parser::new(StreamType::Request);

    assert!(match p.parse(&mut h, b"GET /path H") {
        Ok(Success::Eof(_)) => true,
        _ => false
    });

    assert_eq!(p.get_state(), State::RequestHttp2);

    assert!(match p.parse(&mut h, b"T") {
        Ok(Success::Eof(_)) => true,
        _ => false
    });

    assert_eq!(p.get_state(), State::RequestHttp3);

    assert!(match p.parse(&mut h, b"T") {
        Ok(Success::Eof(_)) => true,
        _ => false
    });

    assert_eq!(p.get_state(), State::RequestHttp4);

    assert!(match p.parse(&mut h, b"P") {
        Ok(Success::Eof(_)) => true,
        _ => false
    });

    assert_eq!(p.get_state(), State::RequestHttp5);

    assert!(match p.parse(&mut h, b"/") {
        Ok(Success::Eof(_)) => true,
        _ => false
    });

    assert_eq!(p.get_state(), State::RequestVersionMajor);
}

#[test]
fn request_http_invalid_byte() {
    let mut h = H{};
    let mut p = Parser::new(StreamType::Request);

    assert!(match p.parse(&mut h, b"GET /path HTT@/") {
        Err(ParserError::Version(_)) => true,
        _ => false
    });

    assert_eq!(p.get_state(), State::Dead);
}
