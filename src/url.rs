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

//! URL handling functions.
//!
//! This module provides hex decoding, hex encoding, query string parsing, and URL parsing.

use byte::is_encoded;
use std::fmt;

/// Invalid hex sequence.
pub const ERR_DECODING_HEX: &'static str = "Invalid hex sequence";

/// Invalid query string parameter field.
pub const ERR_PARAM_FIELD: &'static str = "Invalid query string parameter field";

/// Invalid query string parameter value.
pub const ERR_PARAM_VALUE: &'static str = "Invalid query string parameter value";

/// Invalid URL fragment.
pub const ERR_URL_FRAGMENT: &'static str = "Invalid URL fragment";

/// Invalid URL host.
pub const ERR_URL_HOST: &'static str = "Invalid URL host";

/// Invalid URL path.
pub const ERR_URL_PATH: &'static str = "Invalid URL path";

/// Invalid URL port.
pub const ERR_URL_PORT: &'static str = "Invalid URL port";

/// Invalid query string.
pub const ERR_URL_QUERY_STRING: &'static str = "Invalid query string";

/// Invalid URL scheme.
pub const ERR_URL_SCHEME: &'static str = "Invalid URL scheme";

// -------------------------------------------------------------------------------------------------

/// Hex decoding error messages.
#[derive(Clone,Copy,PartialEq)]
pub enum DecodingError {
    /// Invalid hex sequence.
    Hex(&'static str, u8)
}

impl fmt::Display for DecodingError {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            DecodingError::Hex(msg, byte) => {
                write!(formatter, "{} at byte '{}'", msg, byte as char)
            }
        }
    }
}

// -------------------------------------------------------------------------------------------------

/// Query string parameter error messages.
#[derive(Clone,Copy,PartialEq)]
pub enum ParamError {
    /// Invalid parameter field.
    Field(&'static str, u8),

    /// Invalid parameter value.
    Value(&'static str, u8),
}

impl fmt::Display for ParamError {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            ParamError::Field(msg, byte) => {
                write!(formatter, "{} at byte '{}'", msg, byte as char)
            },
            ParamError::Value(msg, byte) => {
                write!(formatter, "{} at byte '{}'", msg, byte as char)
            }
        }
    }
}

// -------------------------------------------------------------------------------------------------

/// URL error messages.
#[derive(Clone,Copy,PartialEq)]
pub enum UrlError {
    /// Invalid fragment.
    Fragment(&'static str, u8),

    /// Invalid host.
    Host(&'static str, u8),

    /// Invalid path.
    Path(&'static str, u8),

    /// Invalid port.
    Port(&'static str, u8),

    /// Invalid query string.
    QueryString(&'static str, u8),

    /// Invalid scheme.
    Scheme(&'static str, u8)
}

impl fmt::Display for UrlError {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            UrlError::Fragment(msg, byte) => {
                write!(formatter, "{} at byte '{}'", msg, byte as char)
            },
            UrlError::Host(msg, byte) => {
                write!(formatter, "{} at byte '{}'", msg, byte as char)
            },
            UrlError::Path(msg, byte) => {
                write!(formatter, "{} at byte '{}'", msg, byte as char)
            },
            UrlError::Port(msg, byte) => {
                write!(formatter, "{} at byte '{}'", msg, byte as char)
            },
            UrlError::QueryString(msg, byte) => {
                write!(formatter, "{} at byte '{}'", msg, byte as char)
            },
            UrlError::Scheme(msg, byte) => {
                write!(formatter, "{} at byte '{}'", msg, byte as char)
            }
        }
    }
}

// -------------------------------------------------------------------------------------------------

/// Type that handles query string parameter events.
#[allow(unused_variables)]
pub trait ParamHandler {
    /// Callback that is executed when parsing a parameter field has completed.
    ///
    /// This may be executed multiple times in order to supply the entire parameter field.
    ///
    /// Returns `true` when parsing should continue. Otherwise `false`.
    fn on_param_field(&mut self, field: &[u8]) -> bool {
        true
    }

    /// Callback that is executed when parsing a parameter value has completed.
    ///
    /// This may be executed multiple times in order to supply the entire parameter value.
    ///
    /// Returns `true` when parsing should continue. Otherwise `false`.
    fn on_param_value(&mut self, value: &[u8]) -> bool {
        true
    }
}

/// Type that handles URL events.
///
/// URL parsing does not validate or decode any part of the URL, so each callback will receive the
/// entire chunk of data necessary to fulfill the segment within one call.
#[allow(unused_variables)]
pub trait UrlHandler {
    /// Callback that is executed when parsing a URL fragment has completed.
    ///
    /// Returns `true` when parsing should continue. Otherwise `false`.
    fn on_url_fragment(&mut self, fragment: &[u8]) -> bool {
        true
    }

    /// Callback that is executed when parsing a URL host has completed.
    ///
    /// Returns `true` when parsing should continue. Otherwise `false`.
    fn on_url_host(&mut self, host: &[u8]) -> bool {
        true
    }

    /// Callback that is executed when parsing a URL path has completed.
    ///
    /// Returns `true` when parsing should continue. Otherwise `false`.
    fn on_url_path(&mut self, path: &[u8]) -> bool {
        true
    }

    /// Callback that is executed when parsing a URL port has completed.
    ///
    /// Returns `true` when parsing should continue. Otherwise `false`.
    fn on_url_port(&mut self, port: u16) -> bool {
        true
    }

    /// Callback that is executed when parsing a URL query string has completed.
    ///
    /// Returns `true` when parsing should continue. Otherwise `false`.
    fn on_url_query_string(&mut self, query_string: &[u8]) -> bool {
        true
    }

    /// Callback that is executed when parsing a URL scheme has completed.
    ///
    /// Returns `true` when parsing should continue. Otherwise `false`.
    fn on_url_scheme(&mut self, scheme: &[u8]) -> bool {
        true
    }
}

// -------------------------------------------------------------------------------------------------

/// Encode a byte into a URL encoded hex sequence *%XX*.
///
/// # Example
///
/// ```
/// use http_box::url::byte_to_hex;
/// use std::str;
///
/// let hex = byte_to_hex(b'\r');
///
/// println!("Hex: {:?}", str::from_utf8(&hex).unwrap());
/// ```
#[inline]
pub fn byte_to_hex(byte: u8) -> [u8; 3] {
    [b'%', b"0123456789ABCDEF"[(byte >> 4) as usize], b"0123456789ABCDEF"[(byte & 0x0F) as usize]]
}

/// URL decode an array of bytes.
///
/// # Example
///
/// ```
/// use http_box::url::{decode, DecodingError};
///
/// match decode(b"Hello,%20world", &mut vec![]) {
///     Ok(ref decoded) => {
///         // decoded is a Vec<u8> of decoded bytes
///     },
///     Err(error) => println!("{}", error)
/// }
/// ```
#[allow(unused_assignments)]
#[inline]
pub fn decode<'a>(bytes: &[u8], into: &'a mut Vec<u8>) -> Result<&'a mut Vec<u8>, DecodingError> {
    // current byte
    let mut byte: u8;

    // byte index we're processing
    let mut byte_index: usize = 0;

    // byte index for the start of the mark
    let mut mark_index: usize;

    // ---------------------------------------------------------------------------------------------

    // collect macro base
    macro_rules! collect_base {
        ($block:block) => ({
            let mut found = false;

            // put byte index back one byte to reflect our start loop index
            byte_index -= 1;

            while byte_index < bytes.len() {
                byte = bytes[byte_index];

                if $block {
                    found       = true;
                    byte_index += 1;

                    break
                }

                byte_index += 1;
            }

            found
        });
    }

    // collect non-control characters until a certain byte is found
    macro_rules! collect_until {
        ($byte1:expr, $byte2:expr, $error:path, $error_msg:expr) => (
            collect_base!({
                if $byte1 == byte || $byte2 == byte {
                    true
                } else if !is_ascii!(byte) || is_control!(byte) {
                    error!($error($error_msg, byte));
                } else {
                    false
                }
            })
        );
    }

    // return an error
    macro_rules! error {
        ($error:expr) => (
            return Err($error);
        );
    }

    // indicates that we have enough bytes to extract them
    macro_rules! has_bytes {
        ($count:expr) => (
            byte_index + $count - 1 < bytes.len();
        );
    }

    // mark the current byte as the first mark byte
    macro_rules! mark {
        () => (
            mark_index = byte_index - 1;
        );
    }

    // get the marked bytes
    macro_rules! marked_bytes {
        () => (
            &bytes[mark_index..byte_index]
        );

        ($forget:expr) => (
            &bytes[mark_index..byte_index - $forget]
        );
    }

    // skip to the next byte
    macro_rules! next {
        () => ({
            if byte_index == bytes.len() {
                return Ok(into);
            }

            byte        = bytes[byte_index];
            byte_index += 1;

            true
        });
    }

    // ---------------------------------------------------------------------------------------------

    while next!() {
        mark!();

        if collect_until!(b'%', b'+', DecodingError::Hex, ERR_DECODING_HEX) {
            if byte == b'%' {
                into.extend_from_slice(marked_bytes!(1));

                if !has_bytes!(2) {
                    error!(DecodingError::Hex(ERR_DECODING_HEX, byte));
                }

                mark!();
                next!();
                next!();

                match hex_to_byte(marked_bytes!()) {
                    Some(byte) => {
                        into.push(byte);
                    },
                    _ => {
                        error!(DecodingError::Hex(ERR_DECODING_HEX, byte));
                    }
                }
            } else {
                into.extend_from_slice(marked_bytes!(1));
                into.push(b' ');
            }
        } else {
            into.extend_from_slice(marked_bytes!());
        }
    }

    Ok(into)
}

/// URL encode an array of bytes.
///
/// # Example
///
/// ```
/// use http_box::url::encode;
/// use std::str;
///
/// let mut encoded = vec![];
///
/// encode(b"Hello, world!", &mut encoded);
///
/// println!("Encoded: {:?}", str::from_utf8(&encoded[..]).unwrap());
/// ```
#[inline]
pub fn encode<'a>(bytes: &[u8], into: &'a mut Vec<u8>) -> &'a mut Vec<u8> {
    // byte index we're processing
    let mut byte_index: usize = 0;

    // byte index for the start of the mark
    let mut mark_index: usize = 0;

    while byte_index < bytes.len() {
        if is_encoded(bytes[byte_index]) {
            into.extend_from_slice(&bytes[mark_index..byte_index]);
            into.extend_from_slice(&byte_to_hex(bytes[byte_index]));

            mark_index = byte_index + 1;
        }

        byte_index += 1;
    }

    into.extend_from_slice(&bytes[mark_index..byte_index]);
    into
}

/// Decode a single 3 byte URL encoded hex sequence *%XX* into a single byte.
///
/// The *hex* slice length must be at least 3 bytes otherwise this function will panic.
///
/// Returns `None` if the hex sequence is invalid.
///
/// # Example
///
/// ```
/// use http_box::url::hex_to_byte;
///
/// match hex_to_byte(b"%20") {
///     Some(_) => { println!("Decoded a space"); }
///     None    => { println!("Decoding error"); }
/// }
/// ```
#[inline]
pub fn hex_to_byte(hex: &[u8]) -> Option<u8> {
    if hex[0] != b'%' {
        return None;
    }

    let byte: u8 = if is_digit!(hex[1]) {
        (hex[1] - b'0') << 4
    } else if b'@' < hex[1] && hex[1] < b'G' {
        (hex[1] - 0x37) << 4
    } else if b'`' < hex[1] && hex[1] < b'g' {
        (hex[1] - 0x57) << 4
    } else {
        return None;
    };

    if is_digit!(hex[2]) {
        Some(byte + (hex[2] - b'0'))
    } else if b'@' < hex[2] && hex[2] < b'G' {
        Some(byte + (hex[2] - 0x37))
    } else if b'`' < hex[2] && hex[2] < b'g' {
        Some(byte + (hex[2] - 0x57))
    } else {
        None
    }
}

/// Parse a query string, and decode parameter fields and values.
///
/// # Example
///
/// ```
/// use http_box::url::{parse_query_string, ParamError, ParamHandler};
///
/// struct Param {
///     field: Vec<u8>,
///     value: Vec<u8>
/// }
///
/// impl ParamHandler for Param {
///     fn on_param_field(&mut self, data: &[u8]) -> bool {
///         self.field.extend_from_slice(data);
///         true
///     }
///
///     fn on_param_value(&mut self, data: &[u8]) -> bool {
///         self.value.extend_from_slice(data);
///         true
///     }
/// }
///
/// fn main() {
///
/// }
/// ```
#[cfg_attr(test, allow(collapsible_if, cyclomatic_complexity))]
#[allow(unused_assignments)]
pub fn parse_query_string(handler: &mut ParamHandler,
                          bytes: &[u8]) -> Result<bool, ParamError> {
    // current byte
    let mut byte: u8;

    // byte index we're processing
    let mut byte_index: usize = 0;

    // byte index for the start of the mark
    let mut mark_index: usize;

    // ---------------------------------------------------------------------------------------------

    // collect macro base
    macro_rules! collect_base {
        ($block:block) => ({
            let mut found = false;

            // put byte index back one byte to reflect our start loop index
            byte_index -= 1;

            while !is_eof!() {
                byte = peek!();

                if $block {
                    found       = true;
                    byte_index += 1;

                    break
                }

                byte_index += 1;
            }

            found
        });
    }

    // collect non-control characters until a certain byte is found
    macro_rules! collect_until {
        ($byte1:expr, $byte2:expr, $byte3:expr, $byte4:expr, $error:path, $error_msg:expr) => (
            collect_base!({
                if $byte1 == byte || $byte2 == byte || $byte3 == byte || $byte4 == byte {
                    true
                } else if !is_ascii!(byte) || is_control!(byte)  {
                    error!($error($error_msg, byte));
                } else {
                    false
                }
            })
        );

        ($byte1:expr, $byte2:expr, $byte3:expr, $error:path, $error_msg:expr) => (
            collect_base!({
                if $byte1 == byte || $byte2 == byte || $byte3 == byte {
                    true
                } else if !is_ascii!(byte) || is_control!(byte)  {
                    error!($error($error_msg, byte));
                } else {
                    false
                }
            })
        );

        ($byte1:expr, $byte2:expr, $error:path, $error_msg:expr) => (
            collect_base!({
                if $byte1 == byte || $byte2 == byte {
                    true
                } else if !is_ascii!(byte) || is_control!(byte) {
                    error!($error($error_msg, byte));
                } else {
                    false
                }
            })
        );
    }

    // return an error
    macro_rules! error {
        ($error:expr) => (
            return Err($error);
        );
    }

    // indicates that we have enough bytes to extract them
    macro_rules! has_bytes {
        ($count:expr) => (
            byte_index + $count - 1 < bytes.len();
        );
    }

    // check end of bytes
    macro_rules! is_eof {
        () => (
            byte_index == bytes.len()
        );
    }

    // mark the current byte as the first mark byte
    macro_rules! mark {
        () => (
            mark_index = byte_index - 1;
        );
    }

    // get the marked bytes
    macro_rules! marked_bytes {
        () => (
            &bytes[mark_index..byte_index]
        );

        ($forget:expr) => (
            &bytes[mark_index..byte_index - $forget]
        );
    }

    // skip to the next byte
    macro_rules! next {
        () => ({
            if is_eof!() {
                return Ok(true);
            }

            byte        = peek!();
            byte_index += 1;

            true
        });
    }

    // peek at the next byte
    macro_rules! peek {
        () => (
            bytes[byte_index]
        );

        ($count:expr) => (
            bytes[byte_index + $count - 1]
        )
    }

    // ---------------------------------------------------------------------------------------------

    if !is_eof!() && bytes[byte_index] == b'?' {
        next!();
    }

    while next!() {
        mark!();

        // we check for AMPERSAND here, although we do not use it, so the collection will stop on it
        // and allow us to continue with a proper parameter next loop
        if collect_until!(b'=', b'%', b'&', b'+', ParamError::Field, ERR_PARAM_FIELD) {
            if byte == b'=' {
                if !handler.on_param_field(marked_bytes!(1)) {
                    break;
                }

                // parse the value
                while next!() {
                    mark!();

                    if collect_until!(b'%', b'&', b'+', ParamError::Value, ERR_PARAM_VALUE) {
                        if byte == b'%' {
                            if !handler.on_param_value(marked_bytes!(1)) {
                                break;
                            }

                            if !has_bytes!(2) {
                                error!(ParamError::Value(ERR_PARAM_VALUE, byte));
                            }

                            mark!();
                            next!();
                            next!();

                            match hex_to_byte(marked_bytes!()) {
                                Some(byte) => {
                                    if !handler.on_param_value(&[byte]) {
                                        return Ok(false);
                                    }
                                },
                                _ => {
                                    error!(ParamError::Value(ERR_PARAM_VALUE, byte));
                                }
                            }
                        } else if byte == b'+' {
                            if !handler.on_param_value(marked_bytes!(1))
                            || !handler.on_param_value(b" ") {
                                return Ok(false);
                            }
                        } else {
                            if !handler.on_param_value(marked_bytes!(1)) {
                                return Ok(false);
                            }

                            break;
                        }
                    } else {
                        if !handler.on_param_value(marked_bytes!()) {
                            return Ok(false);
                        }

                        break;
                    }
                }
            } else if byte == b'%' {
                if !handler.on_param_field(marked_bytes!(1)) {
                    break;
                }

                if !has_bytes!(2) {
                    error!(ParamError::Field(ERR_PARAM_FIELD, byte));
                }

                mark!();
                next!();
                next!();

                match hex_to_byte(marked_bytes!()) {
                    Some(byte) => {
                        if !handler.on_param_field(&[byte]) {
                            return Ok(false);
                        }
                    },
                    _ => {
                        error!(ParamError::Field(ERR_PARAM_FIELD, byte));
                    }
                }
            } else if byte == b'+' {
                if !handler.on_param_field(marked_bytes!(1))
                || !handler.on_param_field(b" ") {
                    return Ok(false);
                }
            }
        }
    }

    Ok(true)
}

/// Parse a URL.
///
/// This function steps through expected delimiters and feeds back slices of data to callback
/// functions. The only thing that it verifies is that each byte is a 7-bit ASCII compatible byte,
/// and not a control character. The optional port is converted into a u16 if provided.
///
/// This function does not decode any individual part of the URL, nor does it validate the format
/// of any individual part such as the host/ip or path.
///
/// # Example
///
/// ```
/// use http_box::url::{parse_url, UrlError, UrlHandler};
/// use std::str;
///
/// struct Url {
///     fragment:     Vec<u8>,
///     host:         Vec<u8>,
///     path:         Vec<u8>,
///     port:         u16,
///     query_string: Vec<u8>,
///     scheme:       Vec<u8>
/// }
///
/// impl UrlHandler for Url {
///     fn on_url_fragment(&mut self, data: &[u8]) -> bool {
///         self.fragment.extend_from_slice(data);
///         true
///     }
///
///     fn on_url_host(&mut self, data: &[u8]) -> bool {
///         self.host.extend_from_slice(data);
///         true
///     }
///
///     fn on_url_path(&mut self, data: &[u8]) -> bool {
///         self.path.extend_from_slice(data);
///         true
///     }
///
///     fn on_url_port(&mut self, data: u16) -> bool {
///         self.port = data;
///         true
///     }
///
///     fn on_url_query_string(&mut self, data: &[u8]) -> bool {
///         self.query_string.extend_from_slice(data);
///         true
///     }
///
///     fn on_url_scheme(&mut self, data: &[u8]) -> bool {
///         self.scheme.extend_from_slice(data);
///         true
///     }
/// }
///
/// fn main() {
///     let mut url = Url{ fragment: Vec::new(),
///                        host: Vec::new(),
///                        path: Vec::new(),
///                        port: 0,
///                        query_string: Vec::new(),
///                        scheme: Vec::new() };
///
///     match parse_url(&mut url, b"http://www.host.com:80/path?query_string#fragment") {
///         Ok(_) => {
///             println!("Scheme:       {:?}", str::from_utf8(&url.scheme[..]).unwrap());
///             println!("Host:         {:?}", str::from_utf8(&url.host[..]).unwrap());
///             println!("Port:         {}", url.port);
///             println!("Path:         {:?}", str::from_utf8(&url.path[..]).unwrap());
///             println!("Query String: {:?}", str::from_utf8(&url.query_string[..]).unwrap());
///             println!("Fragment:     {:?}", str::from_utf8(&url.fragment[..]).unwrap());
///         },
///         Err(error) => {
///             println!("{}", error);
///         }
///     }
/// }
/// ```
#[cfg_attr(test, allow(cyclomatic_complexity))]
#[allow(unused_assignments)]
pub fn parse_url(handler: &mut UrlHandler, bytes: &[u8]) -> Result<bool, UrlError> {
    // current byte
    let mut byte: u8;

    // byte index we're processing
    let mut byte_index: usize = 0;

    // byte index for the start of the mark
    let mut mark_index: usize;

    // ---------------------------------------------------------------------------------------------

    // collect non-control characters until end of bytes
    macro_rules! collect_all {
        ($error:path, $error_msg:expr) => (
            collect_base!({
                if !is_ascii!(byte) || is_control!(byte) {
                    error!($error($error_msg, byte));
                }

                false
            })
        );
    }

    // collect macro base
    macro_rules! collect_base {
        ($block:block) => ({
            let mut found = false;

            // put byte index back one byte to reflect our start loop index
            byte_index -= 1;

            while !is_eof!() {
                byte = peek!();

                if $block {
                    found       = true;
                    byte_index += 1;

                    break
                }

                byte_index += 1;
            }

            found
        });
    }

    // collect a digit
    macro_rules! collect_digit {
        ($byte:expr, $digit:expr, $max:expr, $error:path, $error_msg:expr) => (
            collect_base!({
                if is_digit!(byte) {
                    $digit *= 10;
                    $digit += byte as u32 - b'0' as u32;

                    if $digit > $max {
                        error!($error($error_msg, byte));
                    }

                    false
                } else if $byte == byte {
                    true
                } else {
                    error!($error($error_msg, byte));
                }
            })
        );
    }

    // collect non-control characters until a certain byte is found
    macro_rules! collect_until {
        ($byte1:expr, $byte2:expr, $error:path, $error_msg:expr) => (
            collect_base!({
                if $byte1 == byte || $byte2 == byte {
                    true
                } else if !is_ascii!(byte) || is_control!(byte)  {
                    error!($error($error_msg, byte));
                } else {
                    false
                }
            })
        );

        ($byte:expr, $error:path, $error_msg:expr) => (
            collect_base!({
                if $byte == byte {
                    true
                } else if !is_ascii!(byte) || is_control!(byte)  {
                    error!($error($error_msg, byte));
                } else {
                    false
                }
            })
        );
    }

    // return an error
    macro_rules! error {
        ($error:expr) => (
            return Err($error);
        );
    }

    // collect a specific number of bytes
    macro_rules! jump {
        // collect an exact amount
        ($count:expr) => (
            byte_index += $count;
            byte        = bytes[byte_index-1];
        );
    }

    // indicates that we have enough bytes to extract them
    macro_rules! has_bytes {
        ($count:expr) => (
            byte_index + $count - 1 < bytes.len();
        );
    }

    // check end of bytes
    macro_rules! is_eof {
        () => (
            byte_index == bytes.len()
        );
    }

    // mark the current byte as the first mark byte
    macro_rules! mark {
        () => (
            mark_index = byte_index - 1;
        );
    }

    // get the marked bytes
    macro_rules! marked_bytes {
        () => (
            &bytes[mark_index..byte_index]
        );

        ($forget:expr) => (
            &bytes[mark_index..byte_index - $forget]
        );
    }

    // skip to the next byte
    macro_rules! next {
        () => ({
            if is_eof!() {
                return Ok(true);
            }

            byte        = peek!();
            byte_index += 1;

            true
        });
    }

    // peek at the next byte
    macro_rules! peek {
        () => (
            bytes[byte_index]
        );

        ($count:expr) => (
            bytes[byte_index + $count - 1]
        )
    }

    // peek at a chunk of bytes starting with the current byte
    macro_rules! peek_chunk {
        ($count:expr) => (
            &bytes[byte_index - 1..byte_index + $count - 1]
        );
    }

    // replay the current byte
    macro_rules! replay {
        () => (
            byte_index -= 1;
            byte        = bytes[byte_index - 1];
        );
    }

    // ---------------------------------------------------------------------------------------------

    if !is_eof!() && bytes[byte_index] != b'/' {
        // scheme
        next!();
        mark!();

        if collect_until!(b':', UrlError::Scheme, ERR_URL_SCHEME) {
            if !handler.on_url_scheme(marked_bytes!(1)) {
                return Ok(false);
            }
        } else {
            error!(UrlError::Scheme(ERR_URL_SCHEME, byte));
        }

        // discard ://
        if has_bytes!(2) && b"://" == peek_chunk!(3) {
            jump!(2);
        } else {
            error!(UrlError::Scheme(ERR_URL_SCHEME, byte));
        }

        // host
        next!();
        mark!();

        if collect_until!(b'/', b':', UrlError::Host, ERR_URL_HOST) {
            if !handler.on_url_host(marked_bytes!(1)) {
                return Ok(false);
            }

            if byte == b':' {
                let mut port: u32 = 0;

                next!();

                if collect_digit!(b'/', port, 65535, UrlError::Port, ERR_URL_PORT) {
                    if !handler.on_url_port(port as u16) {
                        return Ok(false);
                    }
                } else {
                    error!(UrlError::Port(ERR_URL_PORT, byte));
                }
            }

            replay!();
        } else {
            error!(UrlError::Host(ERR_URL_HOST, byte));
        }
    }

    // path
    while next!() {
        mark!();

        if collect_until!(b'?', b'#', UrlError::Path, ERR_URL_PATH) {
            if !handler.on_url_path(marked_bytes!(1)) {
                return Ok(false);
            }

            break;
        } else {
            handler.on_url_path(marked_bytes!());

            return Ok(true);
        }
    }

    // query string
    if byte == b'?' {
        next!();
        mark!();

        if collect_until!(b'#', UrlError::QueryString, ERR_URL_QUERY_STRING) {
            if !handler.on_url_query_string(marked_bytes!(1)) {
                return Ok(false);
            }
        } else {
            handler.on_url_query_string(marked_bytes!());

            return Ok(true);
        }
    }

    // fragment
    if byte == b'#' {
        next!();
        mark!();

        collect_all!(UrlError::Fragment, ERR_URL_FRAGMENT);

        handler.on_url_fragment(marked_bytes!());
    }

    Ok(true)
}
