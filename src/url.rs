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

use byte::hex_to_byte;

use std::{ fmt,
           str };

// -------------------------------------------------------------------------------------------------
// STREAM MACROS
// -------------------------------------------------------------------------------------------------

// Collect digits as a single numerical value.
macro_rules! collect_digits {
    ($stream:expr, $stream_index:expr, $digit:expr, $max:expr, $byte_error:expr,
     $eof_block:block) => ({
        let mut byte;

        loop {
            if is_eof!($stream, $stream_index) {
                $eof_block

                exit_ok!($stream_index);
            }

            byte = next!($stream, $stream_index);

            if is_digit!(byte) {
                $digit *= 10;
                $digit += (byte - b'0') as u32;

                if $digit > $max {
                    exit_error!($byte_error(byte));
                }
            } else {
                break;
            }
        }

        byte
    });
}

// Collect all visible 7-bit bytes, which is any non-control byte with the exception of space.
macro_rules! collect_visible {
    ($stream:expr, $stream_index:expr, $stop1:expr, $stop2:expr, $stop3:expr, $stop4:expr,
     $byte_error:expr, $eof_block:block) => ({
        let mut byte;

        loop {
            if is_eof!($stream, $stream_index) {
                $eof_block

                exit_ok!($stream_index);
            }

            byte = next!($stream, $stream_index);

            if $stop1 == byte || $stop2 == byte || $stop3 == byte || $stop4 == byte {
                break;
            } else if is_non_visible!(byte) {
                exit_error!($byte_error(byte));
            }
        }

        byte
    });

    ($stream:expr, $stream_index:expr, $stop1:expr, $stop2:expr, $stop3:expr, $byte_error:expr,
     $eof_block:block) => ({
        let mut byte;

        loop {
            if is_eof!($stream, $stream_index) {
                $eof_block

                exit_ok!($stream_index);
            }

            byte = next!($stream, $stream_index);

            if $stop1 == byte || $stop2 == byte || $stop3 == byte {
                break;
            } else if is_non_visible!(byte) {
                exit_error!($byte_error(byte));
            }
        }

        byte
    });

    ($stream:expr, $stream_index:expr, $stop1:expr, $stop2:expr, $byte_error:expr,
     $eof_block:block) => ({
        let mut byte;

        loop {
            if is_eof!($stream, $stream_index) {
                $eof_block

                exit_ok!($stream_index);
            }

            byte = next!($stream, $stream_index);

            if $stop1 == byte || $stop2 == byte {
                break;
            } else if is_non_visible!(byte) {
                exit_error!($byte_error(byte));
            }
        }

        byte
    });

    ($stream:expr, $stream_index:expr, $stop:expr, $byte_error:expr, $eof_block:block) => ({
        let mut byte;

        loop {
            if is_eof!($stream, $stream_index) {
                $eof_block

                exit_ok!($stream_index);
            }

            byte = next!($stream, $stream_index);

            if $stop == byte {
                break;
            } else if is_non_visible!(byte) {
                exit_error!($byte_error(byte));
            }
        }

        byte
    });

    ($stream:expr, $stream_index:expr, $byte_error:expr, $eof_block:block) => ({
        let mut byte;

        loop {
            if is_eof!($stream, $stream_index) {
                $eof_block

                exit_ok!($stream_index);
            }

            byte = next!($stream, $stream_index);

            if is_non_visible!(byte) {
                exit_error!($byte_error(byte));
            }
        }

        byte
    });
}

// Exit with an error.
macro_rules! exit_error {
    ($error:expr) => ({
        return Err($error);
    });
}

// Exit with OK status if EOF.
macro_rules! exit_if_eof {
    ($stream:expr, $stream_index:expr) => (
        if is_eof!($stream, $stream_index) {
            return Ok($stream_index);
        }
    );
}

// Exit with OK status.
macro_rules! exit_ok {
    ($stream_index:expr) => ({
        return Ok($stream_index);
    });
}

// Find a pattern within a stream.
macro_rules! find {
    ($stream:expr, $pattern:expr) => ({
        let mut index = None;

        'outer:
        for s in 0..$stream.len() {
            for p in 0..$pattern.len() {
                if $stream.len() <= s + p || $pattern[p] != $stream[s + p] {
                    break;
                } else if $pattern.len() == p + 1 {
                    index = Some(s);

                    break 'outer;
                }
            }
        }

        index
    });
}

// Indicates that an alphabetical character exists within the stream.
macro_rules! has_alpha {
    ($stream:expr) => (
        if is_alpha!($stream[0]) {
            true
        } else {
            let mut found = false;

            for n in &$stream {
                if is_alpha!(*n) {
                    found = true;
                    break;
                }
            }

            found
        }
    );
}

// Indicates that a specified amount of bytes are available.
macro_rules! has_bytes {
    ($stream:expr, $stream_index:expr, $length:expr) => (
        $stream_index + $length <= $stream.len()
    );
}

// Indicates that a byte is alphabetical.
macro_rules! is_alpha {
    ($byte:expr) => ({
        ($byte > 64 && $byte < 91) ||
        ($byte > 96 && $byte < 123)
    });
}

// Indicates that a byte is a digit.
macro_rules! is_digit {
    ($byte:expr) => ({
        $byte > 47 && $byte < 58
    });
}

// Indicates that we're at the end of the stream.
macro_rules! is_eof {
    ($stream:expr, $stream_index:expr) => (
        $stream_index == $stream.len()
    );
}

// Jump a specified amount of bytes.
macro_rules! jump_bytes {
    ($stream_index:expr, $length:expr) => ({
        $stream_index += $length;
    });
}

// Advance the stream one byte.
macro_rules! next {
    ($stream:expr, $stream_index:expr) => ({
        $stream_index += 1;

        $stream[$stream_index - 1]
    });
}

// -------------------------------------------------------------------------------------------------

/// Decoding errors.
pub enum DecodeError {
    /// Invalid byte.
    Byte(u8),

    /// Invalid hex sequence.
    HexSequence(u8)
}

impl fmt::Display for DecodeError {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            DecodeError::Byte(x) => {
                write!(formatter, "Invalid byte at {}", x)
            },
            DecodeError::HexSequence(x) => {
                write!(formatter, "Invalid hex sequence at {}", x)
            }
        }
    }
}

// -------------------------------------------------------------------------------------------------

/// Host types.
pub enum Host<'a> {
    /// Hostname host.
    Hostname(&'a [u8]),

    /// IPv4 host.
    IPv4(&'a [u8]),

    /// IPv6 host.
    IPv6(&'a [u8])
}

impl<'a> fmt::Debug for Host<'a> {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Host::Hostname(x) => {
                write!(formatter, "Hostname({})", str::from_utf8(x).unwrap())
            },
            Host::IPv4(x) => {
                write!(formatter, "IPv4({})", str::from_utf8(x).unwrap())
            },
            Host::IPv6(x) => {
                write!(formatter, "IPv6({})", str::from_utf8(x).unwrap())
            }
        }
    }
}

impl<'a> fmt::Display for Host<'a> {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Host::Hostname(x) => {
                write!(formatter, "{}", str::from_utf8(x).unwrap())
            },
            Host::IPv4(x) => {
                write!(formatter, "{}", str::from_utf8(x).unwrap())
            },
            Host::IPv6(x) => {
                write!(formatter, "{}", str::from_utf8(x).unwrap())
            }
        }
    }
}

// -------------------------------------------------------------------------------------------------

/// Query errors.
pub enum QueryError {
    /// Invalid query field.
    Field(u8),

    /// Invalid query value.
    Value(u8)
}

impl fmt::Display for QueryError {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            QueryError::Field(x) => {
                write!(formatter, "Invalid query field at {}", x)
            },
            QueryError::Value(x) => {
                write!(formatter, "Invalid query value at {}", x)
            }
        }
    }
}

// -------------------------------------------------------------------------------------------------

/// Query segments.
pub enum QuerySegment<'a> {
    /// Field segment.
    Field(&'a [u8]),

    /// Flush segment.
    Flush,

    /// Value segment.
    Value(&'a [u8])
}

impl<'a> fmt::Debug for QuerySegment<'a> {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            QuerySegment::Field(x) => {
                write!(formatter, "Field({})", str::from_utf8(x).unwrap())
            },
            QuerySegment::Flush => {
                write!(formatter, "Flush")
            },
            QuerySegment::Value(x) => {
                write!(formatter, "Value({})", str::from_utf8(x).unwrap())
            }
        }
    }
}

impl<'a> fmt::Display for QuerySegment<'a> {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            QuerySegment::Field(x) => {
                write!(formatter, "{}", str::from_utf8(x).unwrap())
            },
            QuerySegment::Flush => {
                write!(formatter, "Flush")
            },
            QuerySegment::Value(x) => {
                write!(formatter, "{}", str::from_utf8(x).unwrap())
            }
        }
    }
}

// -------------------------------------------------------------------------------------------------

/// URL errors.
pub enum UrlError {
    /// Invalid authority.
    Authority(u8),

    /// Invalid fragment.
    Fragment(u8),

    /// Invalid host.
    Host(u8),

    /// Invalid path.
    Path(u8),

    /// Invalid port.
    Port(u8),

    /// Invalid query string.
    QueryString(u8),

    /// Invalid scheme.
    Scheme(u8),

    /// Invalid userinfo.
    UserInfo(u8)
}

impl fmt::Display for UrlError {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            UrlError::Authority(x) => {
                write!(formatter, "Invalid authority at {}", x)
            },
            UrlError::Fragment(x) => {
                write!(formatter, "Invalid fragment at {}", x)
            },
            UrlError::Host(x) => {
                write!(formatter, "Invalid host at {}", x)
            },
            UrlError::Path(x) => {
                write!(formatter, "Invalid path at {}", x)
            },
            UrlError::Port(x) => {
                write!(formatter, "Invalid port at {}", x)
            },
            UrlError::QueryString(x) => {
                write!(formatter, "Invalid query string at {}", x)
            },
            UrlError::Scheme(x) => {
                write!(formatter, "Invalid scheme at {}", x)
            },
            UrlError::UserInfo(x) => {
                write!(formatter, "Invalid user information at {}", x)
            }
        }
    }
}

// -------------------------------------------------------------------------------------------------

/// URL segments.
pub enum UrlSegment<'a> {
    /// Fragment segment.
    Fragment(&'a [u8]),

    /// Host segment.
    Host(Host<'a>),

    /// Path segment.
    Path(&'a [u8]),

    /// Port segment.
    Port(u16),

    /// Query string segment.
    QueryString(&'a [u8]),

    /// Scheme segment.
    Scheme(&'a [u8]),

    /// User information segment.
    UserInfo(&'a [u8])
}

impl<'a> fmt::Debug for UrlSegment<'a> {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            UrlSegment::Fragment(x) => {
                write!(formatter, "Fragment({})", str::from_utf8(x).unwrap())
            },
            UrlSegment::Host(ref x) => {
                write!(formatter, "{:?}", *x)
            },
            UrlSegment::Path(x) => {
                write!(formatter, "Path({})", str::from_utf8(x).unwrap())
            },
            UrlSegment::Port(x) => {
                write!(formatter, "Port({})", x)
            },
            UrlSegment::QueryString(x) => {
                write!(formatter, "QueryString({})", str::from_utf8(x).unwrap())
            },
            UrlSegment::Scheme(x) => {
                write!(formatter, "Scheme({})", str::from_utf8(x).unwrap())
            },
            UrlSegment::UserInfo(x) => {
                write!(formatter, "UserInfo({})", str::from_utf8(x).unwrap())
            }
        }
    }
}

impl<'a> fmt::Display for UrlSegment<'a> {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            UrlSegment::Fragment(x) => {
                write!(formatter, "{}", str::from_utf8(x).unwrap())
            },
            UrlSegment::Host(ref x) => {
                write!(formatter, "{}", *x)
            },
            UrlSegment::Path(x) => {
                write!(formatter, "{}", str::from_utf8(x).unwrap())
            },
            UrlSegment::Port(x) => {
                write!(formatter, "{}", x)
            },
            UrlSegment::QueryString(x) => {
                write!(formatter, "{}", str::from_utf8(x).unwrap())
            },
            UrlSegment::Scheme(x) => {
                write!(formatter, "{}", str::from_utf8(x).unwrap())
            },
            UrlSegment::UserInfo(x) => {
                write!(formatter, "{}", str::from_utf8(x).unwrap())
            }
        }
    }
}

// -------------------------------------------------------------------------------------------------

/// Decode a URL encoded stream of bytes.
pub fn decode<F>(bytes: &[u8], mut append_fn: F) -> Result<usize, DecodeError>
where F : FnMut(&[u8]) {
    let mut byte;
    let mut mark_index;
    let mut stream_index = 0;

    loop {
        mark_index = stream_index;

        byte = collect_visible!(bytes, stream_index,
                                b'%', b'+',
                                DecodeError::Byte,
                                {
            if mark_index < stream_index {
                append_fn(&bytes[mark_index..stream_index]);
            }
        });

        if mark_index < stream_index - 1 {
            append_fn(&bytes[mark_index..stream_index - 1]);
        }

        if byte == b'+' {
            append_fn(b" ");
        } else if has_bytes!(bytes, stream_index, 2) {
            match hex_to_byte(&bytes[stream_index..stream_index + 2]) {
                Some(byte) => {
                    jump_bytes!(stream_index, 2);

                    append_fn(&[byte]);
                },
                None => {
                    exit_error!(DecodeError::HexSequence(byte));
                }
            }
        } else {
            exit_error!(DecodeError::HexSequence(byte));
        }
    }
}

/// Parse a query string.
pub fn parse_query_string<F>(query_string: &[u8], mut segment_fn: F) -> Result<usize, QueryError>
where F : FnMut(QuerySegment) {
    let mut byte;
    let mut mark_index;
    let mut stream_index = 0;

    loop {
        // field loop
        loop {
            mark_index = stream_index;

            byte = collect_visible!(query_string, stream_index,
                                    b'%', b'+', b'=', b'&',
                                    QueryError::Field,
                                    {
                if mark_index < stream_index {
                    segment_fn(QuerySegment::Field(&query_string[mark_index..stream_index]));
                }

                segment_fn(QuerySegment::Flush);
            });

            if mark_index < stream_index - 1 {
                segment_fn(QuerySegment::Field(&query_string[mark_index..stream_index - 1]));
            }

            if byte == b'%' {
                if has_bytes!(query_string, stream_index, 2) {
                    match hex_to_byte(&query_string[stream_index..stream_index + 2]) {
                        Some(byte) => {
                            jump_bytes!(stream_index, 2);

                            segment_fn(QuerySegment::Field(&[byte]));
                        },
                        None => {
                            exit_error!(QueryError::Field(byte));
                        }
                    }
                } else {
                    exit_error!(QueryError::Field(byte));
                }
            } else if byte == b'+' {
                segment_fn(QuerySegment::Field(b" "));
            } else if byte == b'=' {
                if stream_index == 1 {
                    // first byte cannot be an equal sign
                    exit_error!(QueryError::Field(byte));
                }

                break;
            } else {
                if stream_index == 1 {
                    // first byte cannot be an ampersand
                    exit_error!(QueryError::Field(byte));
                }

                // field without a value, flush
                segment_fn(QuerySegment::Flush);
            }
        }

        // param loop
        loop {
            mark_index = stream_index;

            byte = collect_visible!(query_string, stream_index,
                                    b'%', b'+', b'=', b'&',
                                    QueryError::Value,
                                    {
                if mark_index < stream_index {
                    segment_fn(QuerySegment::Value(&query_string[mark_index..stream_index]));
                }

                segment_fn(QuerySegment::Flush);
            });

            if mark_index < stream_index - 1 {
                segment_fn(QuerySegment::Value(&query_string[mark_index..stream_index - 1]));
            }

            if byte == b'%' {
                if has_bytes!(query_string, stream_index, 2) {
                    match hex_to_byte(&query_string[stream_index..stream_index + 2]) {
                        Some(byte) => {
                            jump_bytes!(stream_index, 2);

                            segment_fn(QuerySegment::Value(&[byte]));
                        },
                        None => {
                            exit_error!(QueryError::Value(byte));
                        }
                    }
                } else {
                    exit_error!(QueryError::Value(byte));
                }
            } else if byte == b'+' {
                segment_fn(QuerySegment::Value(b" "));
            } else if byte == b'=' {
                // value cannot have an equal sign
                exit_error!(QueryError::Value(byte));
            } else {
                break;
            }
        }
    }
}

/// Parse a URL.
pub fn parse_url<F>(url: &[u8], mut segment_fn: F) -> Result<usize, UrlError>
where F : FnMut(UrlSegment) {
    let mut byte;
    let mut mark_index;
    let mut stream_index = 0;

    if is_eof!(url, stream_index) {
        // nothing to parse, zero bytes
        exit_ok!(0);
    }

    // scheme
    if let Some(index) = find!(url, b"://") {
        if !is_alpha!(url[0]) {
            exit_error!(UrlError::Scheme(url[0]));
        }

        for n in 1..index {
            byte = url[n];

            if !is_alpha!(byte) && !is_digit!(byte)
            && byte != b'+' && byte != b'-' && byte != b'.' {
                exit_error!(UrlError::Scheme(byte));
            }
        }

        segment_fn(UrlSegment::Scheme(&url[0..index]));

        // manually advance stream index since we didn't use a collect_* macro
        jump_bytes!(stream_index, 3);

        // scheme requires at least one additional piece of data
        if is_eof!(url, stream_index) {
            exit_error!(UrlError::Scheme(url[stream_index - 1]));
        }
    }

    // authority
    byte = next!(url, stream_index);

    if byte != b'/' && byte != b'?' && byte != b'#' {
        // rewind the stream index so collect_visible!() can start fresh
        stream_index -= 1;
        mark_index    = stream_index;

        byte = collect_visible!(url, stream_index,
                                b'/', b'?', b'#',
                                UrlError::Authority,
                                {
            match process_authority(&url[mark_index..stream_index], &mut segment_fn) {
                Ok(_) => {
                    return Ok(stream_index);
                },
                error => {
                    return error;
                }
            }
        });

        match process_authority(&url[mark_index..stream_index], &mut segment_fn) {
            Ok(_) => {
            },
            error => {
                return error;
            }
        }
    }

    // path
    if byte == b'/' {
        // it's essential to rewind the stream index so that the initial
        // forward slash is included as part of the path
        stream_index -= 1;
        mark_index    = stream_index;

        byte = collect_visible!(url, stream_index,
                                b'?', b'#',
                                UrlError::Path,
                                {
            segment_fn(UrlSegment::Path(&url[mark_index..stream_index]));
        });

        segment_fn(UrlSegment::Path(&url[mark_index..stream_index - 1]));
    }

    // query string
    if byte == b'?' {
        mark_index = stream_index;

        byte = collect_visible!(url, stream_index,
                                b'#',
                                UrlError::QueryString,
                                {
            segment_fn(UrlSegment::QueryString(&url[mark_index..stream_index]));
        });

        segment_fn(UrlSegment::QueryString(&url[mark_index..stream_index - 1]));
    }

    // fragment
    if byte == b'#' {
        mark_index = stream_index;

        collect_visible!(url, stream_index,
                         UrlError::Fragment,
                         {
            segment_fn(UrlSegment::Fragment(&url[mark_index..stream_index]));
        });
    }

    exit_ok!(stream_index);
}

/// Process a URL authority.
fn process_authority<F>(authority: &[u8], mut segment_fn: F) -> Result<usize, UrlError>
where F : FnMut(UrlSegment) {
    let mut byte;
    let mut mark_index   = 0;
    let mut stream_index = 0;

    // userinfo
    if let Some(index) = find!(authority, b"@") {
        if index == 0 {
            // userinfo can't be empty
            exit_error!(UrlError::UserInfo(authority[index]));
        }

        byte = collect_visible!(authority, stream_index,
                                b'@',
                                UrlError::UserInfo,
                                {
            // this won't occur, since we know @ exists
            exit_error!(UrlError::UserInfo(authority[stream_index]));
        });

        segment_fn(UrlSegment::UserInfo(&authority[mark_index..index]));
    }

    // host
    exit_if_eof!(authority, stream_index);

    mark_index = stream_index;
    byte       = next!(authority, stream_index);

    if byte == b'[' {
        // ipv6 address
        byte = collect_visible!(authority, stream_index,
                                b']',
                                UrlError::Host,
                                {
            // missing closing bracket
            exit_error!(UrlError::Host(authority[stream_index]));
        });

        if !validate_ipv6(&authority[mark_index..stream_index]) {
            exit_error!(UrlError::Host(authority[stream_index - 1]));
        }

        segment_fn(UrlSegment::Host(Host::IPv6(&authority[mark_index..stream_index])));
    } else if has_alpha!(authority[mark_index..]) {
        // hostname
        byte = collect_visible!(authority, stream_index,
                                b'/', b':',
                                UrlError::Host,
                                {
            if !validate_hostname(&authority[mark_index..stream_index]) {
                exit_error!(UrlError::Host(authority[stream_index - 1]));
            }

            segment_fn(UrlSegment::Host(Host::Hostname(&authority[mark_index..stream_index])));
        });

        if !validate_hostname(&authority[mark_index..stream_index]) {
            exit_error!(UrlError::Host(authority[stream_index - 1]));
        }

        segment_fn(UrlSegment::Host(Host::Hostname(&authority[mark_index..stream_index])));
    } else {
        // ipv4 address
        byte = collect_visible!(authority, stream_index,
                                b':',
                                UrlError::Host,
                                {
            if !validate_ipv4(&authority[mark_index..stream_index]) {
                exit_error!(UrlError::Host(authority[stream_index - 1]));
            }

            segment_fn(UrlSegment::Host(Host::IPv4(&authority[mark_index..stream_index])));
        });

        if !validate_ipv4(&authority[mark_index..stream_index]) {
            exit_error!(UrlError::Host(authority[stream_index - 1]));
        }

        segment_fn(UrlSegment::Host(Host::IPv4(&authority[mark_index..stream_index])));
    }

    if byte != b':' {
        // invalid end of host
        exit_error!(UrlError::Host(authority[stream_index]));
    }

    // port
    exit_if_eof!(authority, stream_index);

    mark_index = stream_index;

    let mut port = 0;

    collect_digits!(authority, stream_index,
                    port, 65535,
                    UrlError::Port,
                    {
        segment_fn(UrlSegment::Port(port as u16));
    });

    exit_error!(UrlError::Port(authority[stream_index]));
}

/// Validate a hostname.
pub fn validate_hostname(hostname: &[u8]) -> bool {
    true
}

/// Validate a IPv4 address.
pub fn validate_ipv4(ipv4: &[u8]) -> bool {
    true
}

/// Validate a IPv6 address.
pub fn validate_ipv6(ipv6: &[u8]) -> bool {
    true
}
