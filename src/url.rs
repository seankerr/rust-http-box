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
use byte_slice::ByteStream;

use std::{ fmt,
           str };

// -------------------------------------------------------------------------------------------------

// Exit with Ok status.
macro_rules! exit_ok {
    ($context:expr) => ({
        return Ok($context.stream_index);
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

impl fmt::Debug for DecodeError {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            DecodeError::Byte(x) => {
                write!(formatter, "DecodeError::Byte(Invalid byte at {})", x)
            },
            DecodeError::HexSequence(x) => {
                write!(formatter, "DecodeError::HexSequence(Invalid hex sequence at {})", x)
            }
        }
    }
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
                write!(formatter, "Host::Hostname({})", str::from_utf8(x).unwrap())
            },
            Host::IPv4(x) => {
                write!(formatter, "Host::IPv4({})", str::from_utf8(x).unwrap())
            },
            Host::IPv6(x) => {
                write!(formatter, "Host::IPv6({})", str::from_utf8(x).unwrap())
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

impl fmt::Debug for QueryError {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            QueryError::Field(x) => {
                write!(formatter, "QueryError::Field(Invalid query field at {})", x)
            },
            QueryError::Value(x) => {
                write!(formatter, "QueryError::Value(Invalid query value at {})", x)
            }
        }
    }
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
                write!(formatter, "QuerySegment::Field({})", str::from_utf8(x).unwrap())
            },
            QuerySegment::Flush => {
                write!(formatter, "QuerySegment::Flush")
            },
            QuerySegment::Value(x) => {
                write!(formatter, "QuerySegment::Value({})", str::from_utf8(x).unwrap())
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

    /// Invalid query.
    Query(u8),

    /// Invalid scheme.
    Scheme(u8),

    /// Invalid userinfo.
    UserInfo(u8)
}

impl fmt::Debug for UrlError {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            UrlError::Authority(x) => {
                write!(formatter, "UrlError::Authority(Invalid authority at {})", x)
            },
            UrlError::Fragment(x) => {
                write!(formatter, "UrlError::Fragment(Invalid fragment at {})", x)
            },
            UrlError::Host(x) => {
                write!(formatter, "UrlError::Host(Invalid host at {})", x)
            },
            UrlError::Path(x) => {
                write!(formatter, "UrlError::Path(Invalid path at {})", x)
            },
            UrlError::Port(x) => {
                write!(formatter, "UrlError::Port(Invalid port at {})", x)
            },
            UrlError::Query(x) => {
                write!(formatter, "UrlError::Query(Invalid query at {})", x)
            },
            UrlError::Scheme(x) => {
                write!(formatter, "UrlError::Scheme(Invalid scheme at {})", x)
            },
            UrlError::UserInfo(x) => {
                write!(formatter, "UrlError::UserInfo(Invalid user information at {})", x)
            }
        }
    }
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
            UrlError::Query(x) => {
                write!(formatter, "Invalid query at {}", x)
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

    /// Query segment.
    Query(&'a [u8]),

    /// Scheme segment.
    Scheme(&'a [u8]),

    /// User information segment.
    UserInfo(&'a [u8])
}

impl<'a> fmt::Debug for UrlSegment<'a> {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            UrlSegment::Fragment(x) => {
                write!(formatter, "UrlSegment::Fragment({})", str::from_utf8(x).unwrap())
            },
            UrlSegment::Host(ref x) => {
                write!(formatter, "UrlSegment::Host({:?})", *x)
            },
            UrlSegment::Path(x) => {
                write!(formatter, "UrlSegment::Path({})", str::from_utf8(x).unwrap())
            },
            UrlSegment::Port(x) => {
                write!(formatter, "UrlSegment::Port({})", x)
            },
            UrlSegment::Query(x) => {
                write!(formatter, "UrlSegment::Query({})", str::from_utf8(x).unwrap())
            },
            UrlSegment::Scheme(x) => {
                write!(formatter, "UrlSegment::Scheme({})", str::from_utf8(x).unwrap())
            },
            UrlSegment::UserInfo(x) => {
                write!(formatter, "UrlSegment::UserInfo({})", str::from_utf8(x).unwrap())
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
            UrlSegment::Query(x) => {
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
    let mut context = ByteStream::new(bytes);

    loop {
        bs_mark!(context);

        collect_visible!(context, DecodeError::Byte,
            // stop on these bytes
               context.byte == b'%'
            || context.byte == b'+',

            // on end-of-stream
            {
               if context.mark_index < context.stream_index {
                    append_fn(bs_slice!(context));
                }

                exit_ok!(context);
            }
        );

        if bs_slice_length!(context) > 1 {
            append_fn(bs_slice_ignore!(context));
        }

        if context.byte == b'+' {
            append_fn(b" ");
        } else if bs_has_bytes!(context, 2) {
            if let Some(byte) = hex_to_byte(bs_peek!(context, 2)) {
                bs_jump!(context, 2);

                append_fn(&[byte]);
            } else {
                return Err(DecodeError::HexSequence(context.byte));
            }
        } else {
            return Err(DecodeError::HexSequence(context.byte));
        }
    }
}

/// Parse a query.
pub fn parse_query<F>(query: &[u8], separator: u8, mut segment_fn: F) -> Result<usize, QueryError>
where F : FnMut(QuerySegment) {
    let mut context = ByteStream::new(query);

    loop {
        // field loop
        loop {
            bs_mark!(context);

            collect_visible!(context, QueryError::Field,
                // stop on these bytes
                   context.byte == b'%'
                || context.byte == b'+'
                || context.byte == b'='
                || context.byte == separator,

                // on end-of-stream
                {
                    if bs_slice_length!(context) > 0 {
                        segment_fn(QuerySegment::Field(bs_slice!(context)));
                    }

                    segment_fn(QuerySegment::Flush);

                    exit_ok!(context);
                }
            );

            if bs_slice_length!(context) > 1 {
                segment_fn(QuerySegment::Field(bs_slice_ignore!(context)));
            }

            if context.byte == b'%' {
                if bs_has_bytes!(context, 2) {
                    if let Some(byte) = hex_to_byte(bs_peek!(context, 2)) {
                        bs_jump!(context, 2);

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
            bs_mark!(context);

            collect_visible!(context, QueryError::Value,
                // stop on these bytes
                   context.byte == b'%'
                || context.byte == b'+'
                || context.byte == b'='
                || context.byte == separator,

                // on end-of-stream
                {
                    if bs_slice_length!(context) > 0 {
                        segment_fn(QuerySegment::Value(bs_slice!(context)));
                    }

                    segment_fn(QuerySegment::Flush);

                    exit_ok!(context);
                }
            );

            if bs_slice_length!(context) > 1 {
                segment_fn(QuerySegment::Value(bs_slice_ignore!(context)));
            }

            if context.byte == b'%' {
                if bs_has_bytes!(context, 2) {
                    if let Some(byte) = hex_to_byte(bs_peek!(context, 2)) {
                        bs_jump!(context, 2);

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

/// Parse an absolute URL or a relative path URL.
pub fn parse_url<F>(url: &[u8], mut segment_fn: F) -> Result<usize, UrlError>
where F : FnMut(UrlSegment) {
    let mut context = ByteStream::new(url);

    if bs_is_eos!(context) {
        exit_ok!(context);
    }

    // check first byte, see if it's /, ?, or #
    bs_next!(context);

    if context.byte != b'/' && context.byte != b'?' && context.byte != b'#' {
        // scheme or relative path
        bs_replay!(context);

        collect_visible!(context, UrlError::Scheme,
            // stop on these bytes
               !is_alpha!(context.byte)
            && !is_digit!(context.byte)
            && context.byte != b'+'
            && context.byte != b'-'
            && context.byte != b'.',

            // on end-of-stream
            {
                // no other data, this is invalid
                return Err(UrlError::Scheme(context.byte));
            }
        );

        // next byte must be a colon if this is a scheme
        if context.byte == b':' {
            // first byte must be alphabetical
            if !is_alpha!(context.stream[context.mark_index]) {
                return Err(UrlError::Scheme(context.stream[context.mark_index]));
            }

            // scheme cannot be end of URL
            if bs_is_eos!(context) {
                return Err(UrlError::Scheme(context.byte));
            }

            segment_fn(UrlSegment::Scheme(bs_slice_ignore!(context)));

            bs_next!(context);

            if is_alpha!(context.byte) {
                // we don't parse URNs
                return Err(UrlError::Scheme(context.byte));
            }
        }
    }

    // authority or absolute path
    if context.byte == b'/' {
        if bs_has_bytes!(context, 1) && bs_peek!(context, 1) == b"/" {
            // authority
            bs_jump!(context, 1);

            try!(parse_url_authority(&mut context, &mut segment_fn));
        }

        if context.byte == b'/' {
            // path
            bs_replay!(context);

            try!(parse_url_path(&mut context, &mut segment_fn));
        }
    }

    // query string
    if context.byte == b'?' {
        try!(parse_url_query(&mut context, &mut segment_fn));
    }

    // fragment
    if context.byte == b'#' {
        try!(parse_url_fragment(&mut context, &mut segment_fn));
    }

    exit_ok!(context);
}

/// Parse a URL authority.
fn parse_url_authority<'a,F>(context: &'a mut ByteStream, mut segment_fn: F)
-> Result<usize, UrlError> where F : FnMut(UrlSegment) {
    if bs_is_eos!(context) {
        return Err(UrlError::Authority(context.byte));
    }

    bs_mark!(context);

    collect_visible!(context, UrlError::Authority,
        // stop on these bytes
           context.byte == b'@'
        || context.byte == b'/'
        || context.byte == b'?'
        || context.byte == b'#',

        // on end-of-stream
        {
            bs_rewind_to!(context, context.mark_index);

            return parse_url_host(context, &mut segment_fn);
        }
    );

    exit_ok!(context);
}

/// Parse a URL fragment.
fn parse_url_fragment<'a,F>(context: &'a mut ByteStream, mut segment_fn: F)
-> Result<usize, UrlError> where F : FnMut(UrlSegment) {
    bs_mark!(context);

    collect_visible!(context, UrlError::Fragment,
        // on end-of-stream
        {
            segment_fn(UrlSegment::Fragment(bs_slice!(context)));

            exit_ok!(context);
        }
    );
}

/// Parse a URL host.
fn parse_url_host<'a,F>(context: &'a mut ByteStream, mut segment_fn: F) -> Result<usize, UrlError>
where F : FnMut(UrlSegment) {
    exit_ok!(context);
}

/// Parse a URL path.
fn parse_url_path<'a,F>(context: &'a mut ByteStream, mut segment_fn: F) -> Result<usize, UrlError>
where F : FnMut(UrlSegment) {
    bs_mark!(context);

    collect_visible!(context, UrlError::Path,
        // stop on these bytes
           context.byte == b'?'
        || context.byte == b'#',

        // on end-of-stream
        {
            segment_fn(UrlSegment::Path(bs_slice!(context)));

            exit_ok!(context);
        }
    );

    segment_fn(UrlSegment::Path(bs_slice_ignore!(context)));

    exit_ok!(context);
}

/// Parse a URL query.
fn parse_url_query<'a,F>(context: &'a mut ByteStream, mut segment_fn: F)
-> Result<usize, UrlError> where F : FnMut(UrlSegment) {
    bs_mark!(context);

    collect_visible!(context, UrlError::Query,
        // stop on these bytes
        context.byte == b'#',

        // on end-of-stream
        {
            segment_fn(UrlSegment::Query(bs_slice!(context)));

            exit_ok!(context);
        }
    );

    segment_fn(UrlSegment::Query(bs_slice_ignore!(context)));

    exit_ok!(context);
}

/*
/// Parse a URL.
pub fn parse_url<F>(url: &[u8], mut segment_fn: F) -> Result<usize, UrlError>
where F : FnMut(UrlSegment) {
    let mut context = ByteStream::new(url);

    if bs_is_eos!(context) {
        exit_ok!(context);
    }

    // scheme
    if let Some(index) = stream_find_pattern!(context, b"://") {
        collect_length!(context, UrlError::Scheme, index,
            // allow these bytes
               is_alpha!(context.byte)
            || is_digit!(context.byte)
            || context.byte == b'+'
            || context.byte == b'-'
            || context.byte == b'.'
        );

        if index == 0 || !is_alpha!(context.stream[context.mark_index]) {
            return Err(UrlError::Scheme(context.stream[0]));
        }

        segment_fn(UrlSegment::Scheme(bs_slice!(context)));

        // skip over ://
        bs_jump!(context, 3);

        if bs_is_eos!(context) {
            exit_ok!(context);
        }
    }

    // authority
    bs_next!(context);

    if context.byte != b'/' && context.byte != b'?' && context.byte != b'#' {
        bs_replay!(context);
        bs_mark!(context);

        collect_visible!(context, UrlError::Authority, {
                try!(process_authority(&mut ByteStream::new(bs_slice!(context)),
                                       &mut segment_fn));

                exit_ok!(context);
            },

            // stop on these bytes
               context.byte == b'/'
            || context.byte == b'?'
            || context.byte == b'#'
        );

        try!(process_authority(&mut ByteStream::new(bs_slice_ignore!(context)),
                               &mut segment_fn));
    }

    // path
    if context.byte == b'/' {
        bs_replay!(context);
        bs_mark!(context);

        collect_visible!(context, UrlError::Path, {
                segment_fn(UrlSegment::Path(bs_slice!(context)));

                exit_ok!(context);
            },

            // stop on these bytes
               context.byte == b'?'
            || context.byte == b'#'
        );

        segment_fn(UrlSegment::Path(bs_slice_ignore!(context)));
    }

    // query string
    if context.byte == b'?' {
        bs_mark!(context);

        collect_visible!(context, UrlError::Query, {
                segment_fn(UrlSegment::Query(bs_slice!(context)));

                exit_ok!(context);
            },

            // stop on these bytes
            context.byte == b'#'
        );

        segment_fn(UrlSegment::Query(bs_slice_ignore!(context)));
    }

    // fragment
    if context.byte == b'#' {
        bs_mark!(context);

        collect_visible!(context, UrlError::Fragment, {
                segment_fn(UrlSegment::Fragment(bs_slice!(context)));

                exit_ok!(context);
            }
        );
    }

    exit_ok!(context);
}

/// Process a URL authority.
fn process_authority<'a,F>(context: &'a mut ByteStream, mut segment_fn: F) -> Result<usize, UrlError>
where F : FnMut(UrlSegment) {
    collect_visible!(context, UrlError::Authority, {
            bs_rewind_to!(context, context.mark_index);
            bs_next!(context);

            return process_host(context, &mut segment_fn);
        },

        // stop on these bytes
        context.byte == b'@'
    );

    if collected_length!(context) < 2 {
        // missing userinfo
        return Err(UrlError::UserInfo(context.byte));
    }

    segment_fn(UrlSegment::UserInfo(bs_slice_ignore!(context)));

    if bs_is_eos!(context) {
        return exit_ok!(context);
    }

    bs_mark!(context);
    bs_next!(context);

    process_host(context, &mut segment_fn)
}

/// Process a host and port.
fn process_host<'a,F>(context: &'a mut ByteStream, mut segment_fn: F) -> Result<usize, UrlError>
where F : FnMut(UrlSegment) {
    if context.byte == b'[' {
        // ipv6 address
        try!(process_ipv6(context, &mut segment_fn));
    } else if stream_count!(context, b'.',
                            !is_digit!(context.byte) && context.byte != b'.') == 3 {
        // ipv4
        bs_replay!(context);

        try!(process_ipv4(context, &mut segment_fn));
    } else {
        // hostname
        bs_replay!(context);

        try!(process_hostname(context, &mut segment_fn));
    }

    if !bs_is_eos!(context) {
        // port
        if context.byte == b':' {
            try!(process_port(context, &mut segment_fn));
        }
    }

    exit_ok!(context);
}

/// Process a hostname.
fn process_hostname<'a,F>(context: &'a mut ByteStream, mut segment_fn: F) -> Result<usize, UrlError>
where F : FnMut(UrlSegment) {
    exit_ok!(context);

    bs_mark!(context);

    let start = context.mark_index;

    loop {
        bs_mark!(context);

        collect_only!(context, UrlError::Host, {
                // make sure first byte is alpha or digit
                if !is_alpha!(context.stream[context.mark_index])
                && !is_digit!(context.stream[context.mark_index]) {
                    return Err(UrlError::Host(context.stream[context.mark_index]));
                }

                segment_fn(UrlSegment::Host(Host::Hostname(bs_slice!(context))));
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

        // make sure first byte is alpha or digit
        if !is_alpha!(context.stream[context.mark_index])
        && !is_digit!(context.stream[context.mark_index]) {
            return Err(UrlError::Host(context.stream[context.mark_index]));
        }

        if context.byte != b'.' {
            break;
        }
    }
}

/// Process an `IPv4` address.
fn process_ipv4<'a,F>(context: &'a mut ByteStream, mut segment_fn: F) -> Result<usize, UrlError>
where F : FnMut(UrlSegment) {
    let mut byte;
    let mut count = 0;
    let     start = context.mark_index;

    loop {
        byte   = 0;
        count += 1;

        bs_mark!(context);

        collect_digits!(context, UrlError::Host, byte, 255, {
                if collected_length!(context) == 0 {
                    // segment contains no data
                    return Err(UrlError::Host(context.byte));
                }

                if count < 4
                || (   collected_length!(context) > 1
                    && context.stream[context.mark_index] == b'0') {
                    // not enough segments, or no leading zeros
                    return Err(UrlError::Host(context.stream[context.mark_index]));
                }

                bs_mark!(context, start);

                segment_fn(UrlSegment::Host(Host::IPv4(bs_slice!(context))));

                exit_ok!(context);
            }
        );

        if collected_length!(context) == 1 {
            // segment contains no data
            return Err(UrlError::Host(context.byte));
        }

        match context.byte {
            b'.' => {
                if collected_length!(context) > 2
                && context.stream[context.mark_index] == b'0' {
                    // no leading zeros
                    return Err(UrlError::Host(context.stream[context.mark_index]));
                }
            },
            b':' => {
                // port
                if count < 4 {
                    // not enough segments
                    return Err(UrlError::Host(context.stream[context.mark_index]));
                }

                bs_mark!(context, start);

                segment_fn(UrlSegment::Host(Host::IPv4(bs_slice_ignore!(context))));

                exit_ok!(context);
            },
            _ => {
                return Err(UrlError::Host(context.byte));
            }
        }
    }
}

/// Process an `IPv6` address.
fn process_ipv6<'a,F>(context: &'a mut ByteStream, mut segment_fn: F) -> Result<usize, UrlError>
where F : FnMut(UrlSegment) {
    Ok(0)
}
*/

/// Process a port.
fn process_port<'a,F>(context: &'a mut ByteStream, mut segment_fn: F) -> Result<usize, UrlError>
where F : FnMut(UrlSegment) {
    let mut port = 0;

    collect_digits!(context, UrlError::Port, port, 65535, {
            segment_fn(UrlSegment::Port(port as u16));

            exit_ok!(context);
        }
    );

    segment_fn(UrlSegment::Port(port as u16));

    exit_ok!(context);
}
