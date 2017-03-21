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

use std::fmt;

/// Parser error messages.
#[derive(Clone,Copy,PartialEq)]
pub enum ParserError {
    /// Invalid chunk extension name on byte `u8`.
    ChunkExtensionName(u8),

    /// Invalid chunk extension value on byte `u8`.
    ChunkExtensionValue(u8),

    /// Invalid chunk length on byte `u8`.
    ChunkLength(u8),

    /// Invalid CRLF sequence on byte `u8`.
    CrlfSequence(u8),

    /// Parsing has failed.
    Dead,

    /// Invalid header name on byte `u8`.
    HeaderName(u8),

    /// Invalid header value on byte `u8`.
    HeaderValue(u8),

    /// Chunk length overflow.
    ChunkLengthOverflow,

    /// Invalid request method on byte `u8`.
    Method(u8),

    /// Invalid multipart data.
    Multipart(u8),

    /// Invalid multipart boundary.
    MultipartBoundary(u8),

    /// Invalid status on byte `u8`.
    Status(u8),

    /// Invalid status code on byte `u8`.
    StatusCode(u8),

    /// Invalid URL character on byte `u8`.
    Url(u8),

    /// Invalid URL encoded name on byte `u8`.
    UrlEncodedName(u8),

    /// Invalid URL encoded value on byte `u8`.
    UrlEncodedValue(u8),

    /// Invalid HTTP version on byte `u8`.
    Version(u8),
}

impl fmt::Debug for ParserError {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            ParserError::ChunkExtensionName(byte) => {
                write!(
                    formatter,
                    "<ParserError::ChunkExtensionName: Invalid chunk extension name on byte {}>",
                    byte
                )
            },
            ParserError::ChunkExtensionValue(byte) => {
                write!(
                    formatter,
                    "<ParserError::ChunkExtensionValue Invalid chunk extension value on byte {}>",
                    byte
                )
            },
            ParserError::ChunkLength(byte) => {
                write!(
                    formatter,
                    "<ParserError::ChunkLength: Invalid chunk length on byte {}>",
                    byte
                )
            },
            ParserError::ChunkLengthOverflow => {
                write!(
                    formatter,
                    "<ParserError::ChunkLengthOverflow: Chunk length overflow>"
                )
            },
            ParserError::CrlfSequence(byte) => {
                write!(
                    formatter,
                    "<ParserError::CrlfSequence: Invalid CRLF sequence on byte {}>",
                    byte
                )
            },
            ParserError::Dead => {
                write!(
                    formatter,
                    "<ParserError::Dead: Parser is dead>"
                )
            },
            ParserError::HeaderName(byte) => {
                write!(
                    formatter,
                    "<ParserError::HeaderName: Invalid header name on byte {}>",
                    byte
                )
            },
            ParserError::HeaderValue(byte) => {
                write!(
                    formatter,
                    "<ParserError::HeaderValue: Invalid header value on byte {}>",
                    byte
                )
            },
            ParserError::Method(byte) => {
                write!(
                    formatter,
                    "<ParserError::Method: Invalid method on byte {}>",
                    byte
                )
            },
            ParserError::Multipart(byte) => {
                write!(
                    formatter,
                    "<ParserError::Multipart: Invalid multipart data on byte {}>",
                    byte
                )
            },
            ParserError::MultipartBoundary(byte) => {
                write!(
                    formatter,
                    "<ParserError::MultipartBoundary: Invalid multipart boundary on byte {}>",
                    byte
                )
            },
            ParserError::Status(byte) => {
                write!(
                    formatter,
                    "<ParserError::Status: Invalid status on byte {}>",
                    byte
                )
            },
            ParserError::StatusCode(byte) => {
                write!(
                    formatter,
                    "<ParserError::StatusCode: Invalid status code on byte {}>",
                    byte
                )
            },
            ParserError::Url(byte) => {
                write!(
                    formatter,
                    "<ParserError::Url: Invalid URL on byte {}>",
                    byte
                )
            },
            ParserError::UrlEncodedName(byte) => {
                write!(
                    formatter,
                    "<ParserError::UrlEncodedName: Invalid URL encoded name on byte {}>",
                    byte
                )
            },
            ParserError::UrlEncodedValue(byte) => {
                write!(
                    formatter,
                    "<ParserError::UrlEncodedValue: Invalid URL encoded value on byte {}>",
                    byte
                )
            },
            ParserError::Version(byte) => {
                write!(
                    formatter,
                    "<ParserError::Version: Invalid HTTP version on byte {}>",
                    byte
                )
            }
        }
    }
}

impl fmt::Display for ParserError {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            ParserError::ChunkExtensionName(byte) => {
                write!(
                    formatter,
                    "<ChunkExtensionName: Invalid chunk extension name on byte {}>",
                    byte
                )
            },
            ParserError::ChunkExtensionValue(byte) => {
                write!(
                    formatter,
                    "<ChunkExtensionValue: Invalid chunk extension value on byte {}>",
                    byte
                )
            },
            ParserError::ChunkLength(byte) => {
                write!(
                    formatter,
                    "<ChunkLength: Invalid chunk length on byte {}>",
                    byte
                )
            },
            ParserError::ChunkLengthOverflow => {
                write!(
                    formatter,
                    "<ChunkLengthOverflow: Chunk length overflow>"
                )
            },
            ParserError::CrlfSequence(byte) => {
                write!(
                    formatter,
                    "<CrlfSequence: Invalid CRLF sequence on byte {}>",
                    byte
                )
            },
            ParserError::Dead => {
                write!(
                    formatter,
                    "<Dead: Parser is dead>"
                )
            },
            ParserError::HeaderName(byte) => {
                write!(
                    formatter,
                    "<HeaderName: Invalid header name on byte {}>",
                    byte
                )
            },
            ParserError::HeaderValue(byte) => {
                write!(
                    formatter,
                    "<HeaderValue: Invalid header value on byte {}>",
                    byte
                )
            },
            ParserError::Method(byte) => {
                write!(
                    formatter,
                    "<Method: Invalid method on byte {}>",
                    byte
                )
            },
            ParserError::Multipart(byte) => {
                write!(
                    formatter,
                    "<Multipart: Invalid multipart data on byte {}>",
                    byte
                )
            },
            ParserError::MultipartBoundary(byte) => {
                write!(
                    formatter,
                    "<MultipartBoundary: Invalid multipart boundary on byte {}>",
                    byte
                )
            },
            ParserError::Status(byte) => {
                write!(
                    formatter,
                    "<Status: Invalid status on byte {}>",
                    byte
                )
            },
            ParserError::StatusCode(byte) => {
                write!(
                    formatter,
                    "<StatusCode: Invalid status code on byte {}>",
                    byte
                )
            },
            ParserError::Url(byte) => {
                write!(
                    formatter,
                    "<Url: Invalid URL on byte {}>",
                    byte
                )
            },
            ParserError::UrlEncodedName(byte) => {
                write!(
                    formatter,
                    "<UrlEncodedName: Invalid URL encoded name on byte {}>",
                    byte
                )
            },
            ParserError::UrlEncodedValue(byte) => {
                write!(
                    formatter,
                    "<UrlEncodedValue: Invalid URL encoded value on byte {}>",
                    byte
                )
            },
            ParserError::Version(byte) => {
                write!(
                    formatter,
                    "<Version: Invalid HTTP version on byte {}>",
                    byte
                )
            }
        }
    }
}
