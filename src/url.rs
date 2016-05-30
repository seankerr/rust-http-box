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
use stream::Context;

use std::{ fmt,
           str };

// -------------------------------------------------------------------------------------------------
// STREAM MACROS
// -------------------------------------------------------------------------------------------------

// Exit with error status.
// Exit with Ok status.
macro_rules! exit_ok {
    ($context:expr) => ({
        return Ok($context.stream_index);
    });
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
    let mut context = Context::new(bytes);

    loop {
        stream_mark!(context);

        stream_collect_visible!(context, DecodeError::Byte, {
               if context.mark_index < context.stream_index {
                    append_fn(stream_collected_bytes!(context));
                }

                exit_ok!(context);
            },

            // stop on these bytes
               context.byte == b'%'
            || context.byte == b'+'
        );

        if context.mark_index < context.stream_index - 1 {
            append_fn(stream_collected_bytes_ignore!(context));
        }

        if context.byte == b'+' {
            append_fn(b" ");
        } else if stream_has_bytes!(context, 2) {
            if let Some(byte) = hex_to_byte(stream_peek!(context, 2)) {
                stream_jump!(context, 2);

                append_fn(&[byte]);
            } else {
                return Err(DecodeError::HexSequence(context.byte));
            }
        } else {
            return Err(DecodeError::HexSequence(context.byte));
        }
    }
}

/// Parse a query string.
pub fn parse_query_string<F>(query_string: &[u8], mut segment_fn: F) -> Result<usize, QueryError>
where F : FnMut(QuerySegment) {
    let mut context = Context::new(query_string);

    loop {
        // field loop
        loop {
            stream_mark!(context);

            stream_collect_visible!(context, QueryError::Field, {
                    if context.mark_index < context.stream_index {
                        segment_fn(QuerySegment::Field(stream_collected_bytes!(context)));
                    }

                    segment_fn(QuerySegment::Flush);

                    exit_ok!(context);
                },

                // stop on these bytes
                   context.byte == b'%'
                || context.byte == b'+'
                || context.byte == b'='
                || context.byte == b'&'
            );

            if context.mark_index < context.stream_index - 1 {
                segment_fn(QuerySegment::Field(stream_collected_bytes_ignore!(context)));
            }

            if context.byte == b'%' {
                if stream_has_bytes!(context, 2) {
                    if let Some(byte) = hex_to_byte(stream_peek!(context, 2)) {
                        stream_jump!(context, 2);

                        segment_fn(QuerySegment::Field(&[byte]));
                    } else {
                        return Err(QueryError::Field(context.byte));
                    }
                } else {
                    return Err(QueryError::Field(context.byte));
                }
            } else if context.byte == b'+' {
                segment_fn(QuerySegment::Field(b" "));
            } else if context.byte == b'=' {
                if context.stream_index == 1 {
                    // first byte cannot be an equal sign
                    return Err(QueryError::Field(context.byte));
                }

                break;
            } else if context.stream_index == 1 {
                // first byte cannot be an ampersand
                return Err(QueryError::Field(context.byte));
            } else {
                // field without a value, flush
                segment_fn(QuerySegment::Flush);
            }
        }

        // param loop
        loop {
            stream_mark!(context);

            stream_collect_visible!(context, QueryError::Value, {
                    if context.mark_index < context.stream_index {
                        segment_fn(QuerySegment::Value(stream_collected_bytes!(context)));
                    }

                    segment_fn(QuerySegment::Flush);

                    exit_ok!(context);
                },

                // stop on these bytes
                   context.byte == b'%'
                || context.byte == b'+'
                || context.byte == b'='
                || context.byte == b'&'
            );

            if context.mark_index < context.stream_index - 1 {
                segment_fn(QuerySegment::Value(stream_collected_bytes_ignore!(context)));
            }

            if context.byte == b'%' {
                if stream_has_bytes!(context, 2) {
                    if let Some(byte) = hex_to_byte(stream_peek!(context, 2)) {
                        stream_jump!(context, 2);

                        segment_fn(QuerySegment::Value(&[byte]));
                    } else {
                        return Err(QueryError::Value(context.byte));
                    }
                } else {
                    return Err(QueryError::Value(context.byte));
                }
            } else if context.byte == b'+' {
                segment_fn(QuerySegment::Value(b" "));
            } else if context.byte == b'=' {
                // value cannot have an equal sign
                return Err(QueryError::Value(context.byte));
            } else {
                segment_fn(QuerySegment::Flush);

                break;
            }
        }
    }
}

/// Parse a URL.
pub fn parse_url<F>(url: &[u8], mut segment_fn: F) -> Result<usize, UrlError>
where F : FnMut(UrlSegment) {
    let mut context = Context::new(url);

    if stream_is_eos!(context) {
        exit_ok!(context);
    }

    // scheme
    if let Some(index) = stream_find_pattern!(context, b"://") {
        stream_collect_length!(context, UrlError::Scheme, index,
            // allow these bytes
               is_alpha!(context.byte)
            || is_digit!(context.byte)
            || context.byte == b'+'
            || context.byte == b'-'
            || context.byte == b'.'
        );

        if !is_alpha!(context.stream[context.mark_index]) {
            // first character must be alphabetical
            return Err(UrlError::Scheme(context.stream[0]));
        }

        segment_fn(UrlSegment::Scheme(stream_collected_bytes!(context)));

        // skip over ://
        stream_jump!(context, 3);

        if stream_is_eos!(context) {
            exit_ok!(context);
        }
    }

    // authority
    stream_next!(context);

    if context.byte != b'/' && context.byte != b'?' && context.byte != b'#' {
        stream_replay!(context);

        try!(process_authority(&mut context, &mut segment_fn));

        if stream_is_eos!(context) {
            exit_ok!(context);
        }

        stream_next!(context);
    }

    // path
    if context.byte == b'/' {
        stream_replay!(context);
        stream_mark!(context);

        stream_collect_visible!(context, UrlError::Path, {
                segment_fn(UrlSegment::Path(stream_collected_bytes!(context)));

                exit_ok!(context);
            },

            // stop on these bytes
               context.byte == b'?'
            || context.byte == b'#'
        );

        segment_fn(UrlSegment::Path(stream_collected_bytes_ignore!(context)));
    }

    // query string
    if context.byte == b'?' {
        stream_mark!(context);

        stream_collect_visible!(context, UrlError::QueryString, {
                segment_fn(UrlSegment::QueryString(stream_collected_bytes!(context)));

                exit_ok!(context);
            },

            // stop on these bytes
            context.byte == b'#'
        );

        segment_fn(UrlSegment::QueryString(stream_collected_bytes_ignore!(context)));
    }

    // fragment
    if context.byte == b'#' {
        stream_mark!(context);

        stream_collect_visible!(context, UrlError::Fragment, {
                segment_fn(UrlSegment::Fragment(stream_collected_bytes!(context)));

                exit_ok!(context);
            }
        );
    }

    exit_ok!(context);
}

/// Process a URL authority.
fn process_authority<'a,F>(context: &'a mut Context, mut segment_fn: F) -> Result<usize, UrlError>
where F : FnMut(UrlSegment) {
    stream_mark!(context);

    stream_collect_visible!(context, UrlError::Authority, {
            stream_rewind_to!(context, context.mark_index);
            stream_next!(context);

            return process_host(context, &mut segment_fn);
        },

        // stop on these bytes
           context.byte == b'/'
        || context.byte == b'?'
        || context.byte == b'#'
        || context.byte == b'@'
    );

    // userinfo
    if context.byte == b'@' {
        if context.stream_index - context.mark_index == 1 {
            // missing userinfo
            return Err(UrlError::UserInfo(context.byte));
        }

        segment_fn(UrlSegment::UserInfo(stream_collected_bytes_ignore!(context)));

        stream_mark!(context);

        stream_collect_visible!(context, UrlError::Authority, {
                stream_rewind_to!(context, context.mark_index);

                return process_host(context, &mut segment_fn);
            },

            // stop on these bytes
               context.byte == b'/'
            || context.byte == b'?'
            || context.byte == b'#'
        );
    }

    if context.stream_index - context.mark_index > 1 {
        stream_rewind_to!(context, context.mark_index);
        stream_next!(context);

        process_host(context, &mut segment_fn)
    } else {
        exit_ok!(context);
    }
}

/// Process a host and port.
fn process_host<'a,F>(context: &'a mut Context, mut segment_fn: F) -> Result<usize, UrlError>
where F : FnMut(UrlSegment) {
    if context.byte == b'[' {
        // ipv6 address
        try!(process_ipv6(context, &mut segment_fn));

        // port
        if context.byte == b':' {
            try!(process_port(context, &mut segment_fn));
        }
    } else {
        // ipv4
        stream_replay!(context);

        try!(process_ipv4(context, &mut segment_fn));

        // port
        if context.byte == b':' {
            try!(process_port(context, &mut segment_fn));
        }
    }

    exit_ok!(context);
}

/// Process a hostname.
fn process_hostname<'a,F>(context: &'a mut Context, mut segment_fn: F) -> Result<usize, UrlError>
where F : FnMut(UrlSegment) {
    exit_ok!(context);
    /*
    stream_mark!(context);

    let start = context.mark_index;

    loop {
        stream_mark!(context);

        stream_collect_only!(context, UrlError::Host, {
                // make sure first byte is alpha
                if !is_alpha!(context.stream[context.mark_index]) {
                    return Err(UrlError::Host(context.stream[context.mark_index]));
                }

                segment_fn(UrlSegment::Host(Host::Hostname(stream_collected_bytes!(context))));
            },

            // stop on these bytes
               context.byte == b'.'
            || context.byte == b':'
            || context.byte == b'/'
            || context.byte == b'?'
            || context.byte == b'#',

            // collect these bytes
               is_alpha!(context.byte)
            || is_digit!(context.byte)
            || context.byte == b'-'
        );

        // make sure first byte is alpha
        if !is_alpha!(context.stream[context.mark_index]) {
            return Err(UrlError::Host(context.stream[context.mark_index]));
        }

        if context.byte != b'.' {
            break;
        }
    }
    */
}

/// Process an `IPv4` address.
fn process_ipv4<'a,F>(context: &'a mut Context, mut segment_fn: F) -> Result<usize, UrlError>
where F : FnMut(UrlSegment) {
    let mut byte;
    let mut count = 0;
    let     start = context.stream_index;

    while count < 4 {
        byte   = 0;
        count += 1;

        stream_mark!(context);

        stream_collect_digits!(context, UrlError::Host, byte, 255, {
                if count < 4 {
                    // less than 4 segments
                    return Err(UrlError::Host(context.byte));
                }

                if context.stream_index - context.mark_index > 1
                && context.stream[context.mark_index] == b'0' {
                    // no leading zeros
                    return Err(UrlError::Host(context.stream[context.mark_index]));
                }

                stream_mark!(context, start);

                segment_fn(UrlSegment::Host(Host::IPv4(stream_collected_bytes!(context))));

                exit_ok!(context);
            }
        );

        if count < 4 && context.byte != b'.' {
            // not a period
            return Err(UrlError::Host(context.byte));
        }

        if context.stream_index - context.mark_index > 2
        && context.stream[context.mark_index] == b'0' {
            // no leading zeros
            return Err(UrlError::Host(context.stream[context.mark_index]));
        }
    }

    stream_mark!(context, start);

    segment_fn(UrlSegment::Host(Host::IPv4(stream_collected_bytes!(context))));

    exit_ok!(context);
}

/// Process an `IPv6` address.
fn process_ipv6<'a,F>(context: &'a mut Context, mut segment_fn: F) -> Result<usize, UrlError>
where F : FnMut(UrlSegment) {
    Ok(0)
}

/// Process a port.
fn process_port<'a,F>(context: &'a mut Context, mut segment_fn: F) -> Result<usize, UrlError>
where F : FnMut(UrlSegment) {
    let mut port = 0;

    stream_collect_digits!(context, UrlError::Port, port, 65535, {
            segment_fn(UrlSegment::Port(port as u16));

            exit_ok!(context);
        }
    );

    segment_fn(UrlSegment::Port(port as u16));

    exit_ok!(context);
}
