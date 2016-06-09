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
use fsm::*;

mod chunk_data;
mod chunk_extension_name;
mod chunk_extension_quoted_value;
mod chunk_extension_value;
mod chunk_size;
mod chunk_trailer;

mod header_field;
mod header_quoted_value;
mod header_value;
mod headers_finished;

/*
mod multipart_boundary;
mod multipart_data;
mod multipart_header;
*/

mod request_method;
mod request_url;
mod request_http;
mod request_version;

mod response_http;
mod response_version;
mod response_status_code;
mod response_status;

mod url_encoded_field;
mod url_encoded_value;

pub fn assert_callback<T: Http1Handler>(parser: &mut Parser<T>, handler: &mut T, stream: &[u8],
                                       state: State, length: usize) {
    assert!(match parser.parse_headers(handler, stream) {
        Ok(Success::Callback(byte_count)) => {
            assert_eq!(byte_count, length);
            assert_eq!(parser.get_state(), state);
            true
        },
        _ => false
    });
}

pub fn assert_eos<T: Http1Handler>(parser: &mut Parser<T>, handler: &mut T, stream: &[u8],
                                  state: State, length: usize) {
    assert!(match parser.parse_headers(handler, stream) {
        Ok(Success::Eos(byte_count)) => {
            assert_eq!(byte_count, length);
            assert_eq!(parser.get_state(), state);
            true
        },
        _ => false
    });
}

pub fn assert_error<T: Http1Handler>(parser: &mut Parser<T>, handler: &mut T, stream: &[u8])
-> Option<ParserError> {
    match parser.parse_headers(handler, stream) {
        Err(error) => {
            assert_eq!(parser.get_state(), State::Dead);
            Some(error)
        },
        _ => {
            assert_eq!(parser.get_state(), State::Dead);
            None
        }
    }
}

pub fn assert_finished<T: Http1Handler>(parser: &mut Parser<T>, handler: &mut T, stream: &[u8],
                                       state: State, length: usize) {
    assert!(match parser.parse_headers(handler, stream) {
        Ok(Success::Finished(byte_count)) => {
            assert_eq!(byte_count, length);
            assert_eq!(parser.get_state(), state);
            true
        },
        _ => false
    });
}

pub fn chunked_assert_callback<T: Http1Handler>(parser: &mut Parser<T>, handler: &mut T,
                                               stream: &[u8], state: State, length: usize) {
    assert!(match parser.parse_chunked(handler, stream) {
        Ok(Success::Callback(byte_count)) => {
            assert_eq!(byte_count, length);
            assert_eq!(parser.get_state(), state);
            true
        },
        _ => false
    });
}

pub fn chunked_assert_eos<T: Http1Handler>(parser: &mut Parser<T>, handler: &mut T, stream: &[u8],
                                          state: State, length: usize) {
    assert!(match parser.parse_chunked(handler, stream) {
        Ok(Success::Eos(byte_count)) => {
            assert_eq!(byte_count, length);
            assert_eq!(parser.get_state(), state);
            true
        },
        _ => false
    });
}

pub fn chunked_assert_error<T: Http1Handler>(parser: &mut Parser<T>, handler: &mut T, stream: &[u8])
-> Option<ParserError> {
    match parser.parse_chunked(handler, stream) {
        Err(error) => {
            assert_eq!(parser.get_state(), State::Dead);
            Some(error)
        },
        _ => {
            assert_eq!(parser.get_state(), State::Dead);
            None
        }
    }
}

pub fn chunked_assert_finished<T: Http1Handler>(parser: &mut Parser<T>, handler: &mut T,
                                               stream: &[u8], state: State, length: usize) {
    assert!(match parser.parse_chunked(handler, stream) {
        Ok(Success::Finished(byte_count)) => {
            assert_eq!(byte_count, length);
            assert_eq!(parser.get_state(), state);
            true
        },
        _ => false
    });
}

pub fn chunked_setup<T:Http1Handler>(parser: &mut Parser<T>, handler: &mut T, stream: &[u8],
                                    state: State) {
    assert!(match parser.parse_chunked(handler, stream) {
        Ok(Success::Eos(length)) => {
            assert_eq!(length, stream.len());
            assert_eq!(parser.get_state(), state);
            true
        },
        _ => false
    });
}

pub fn setup<T:Http1Handler>(parser: &mut Parser<T>, handler: &mut T, stream: &[u8], state: State) {
    assert!(match parser.parse_headers(handler, stream) {
        Ok(Success::Eos(length)) => {
            assert_eq!(length, stream.len());
            assert_eq!(parser.get_state(), state);
            true
        },
        _ => false
    });
}

pub fn url_encoded_assert_callback<T: Http1Handler>(parser: &mut Parser<T>, handler: &mut T,
                                                   stream: &[u8], state: State, length: usize) {
    assert!(match parser.parse_url_encoded(handler, stream) {
        Ok(Success::Callback(byte_count)) => {
            assert_eq!(byte_count, length);
            assert_eq!(parser.get_state(), state);
            true
        },
        _ => false
    });
}

pub fn url_encoded_assert_eos<T: Http1Handler>(parser: &mut Parser<T>, handler: &mut T,
                                              stream: &[u8], state: State, length: usize) {
    assert!(match parser.parse_url_encoded(handler, stream) {
        Ok(Success::Eos(byte_count)) => {
            assert_eq!(byte_count, length);
            assert_eq!(parser.get_state(), state);
            true
        },
        _ => false
    });
}

pub fn url_encoded_assert_error<T: Http1Handler>(parser: &mut Parser<T>, handler: &mut T,
                                                stream: &[u8])
-> Option<ParserError> {
    match parser.parse_url_encoded(handler, stream) {
        Err(error) => {
            assert_eq!(parser.get_state(), State::Dead);
            Some(error)
        },
        _ => {
            assert_eq!(parser.get_state(), State::Dead);
            None
        }
    }
}

pub fn url_encoded_assert_finished<T: Http1Handler>(parser: &mut Parser<T>, handler: &mut T,
                                                   stream: &[u8], state: State, length: usize) {
    assert!(match parser.parse_url_encoded(handler, stream) {
        Ok(Success::Finished(byte_count)) => {
            assert_eq!(byte_count, length);
            assert_eq!(parser.get_state(), state);
            true
        },
        _ => false
    });
}

pub fn url_encoded_setup<T:Http1Handler>(parser: &mut Parser<T>, handler: &mut T, stream: &[u8],
                                        state: State) {
    assert!(match parser.parse_url_encoded(handler, stream) {
        Ok(Success::Eos(length)) => {
            assert_eq!(length, stream.len());
            assert_eq!(parser.get_state(), state);
            true
        },
        _ => false
    });
}
