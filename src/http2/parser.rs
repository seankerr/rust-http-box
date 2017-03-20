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

//! HTTP 2.x parser.

#![allow(dead_code)]

use fsm::{ ParserValue, Success };
use http2::flags::{ FL_PADDED, FL_PRIORITY };
use http2::frame_type::{ FR_CONTINUATION,
                         FR_DATA,
                         FR_GO_AWAY,
                         FR_HEADERS,
                         FR_PING,
                         FR_PRIORITY,
                         FR_PUSH_PROMISE,
                         FR_RST_STREAM,
                         FR_SETTINGS,
                         FR_WINDOW_UPDATE };
use http2::http_handler::HttpHandler;
use http2::parser_state::ParserState;

use byte_slice::ByteStream;
use std::fmt;

// -------------------------------------------------------------------------------------------------
// MACROS
// -------------------------------------------------------------------------------------------------

/// Retrieve the remaining frame payload length sans padding data.
macro_rules! actual_length {
    ($parser:expr) => ({
        payload_length!($parser) - pad_length!($parser)
    });
}

/// Decrease payload length.
macro_rules! dec_payload_length {
    ($parser:expr, $length:expr) => ({
        $parser.bit_data32a = ($parser.bit_data32a & 0xFF)
                            | (($parser.bit_data32a >> 8) - $length) << 8;
    });
}

/// Retrieve a u8.
macro_rules! get_u8 {
    ($context:expr) => ({
        bs_jump!($context, 1);

        $context.stream[$context.stream_index - 1]
    });
}

/// Indicates that a flag is set.
macro_rules! has_flag {
    ($parser:expr, $flag:expr) => (
        $parser.bit_data16a as u8 & $flag == $flag
    );
}

/// Retrieve the pad length.
macro_rules! pad_length {
    ($parser:expr) => ({
        $parser.bit_data32a & 0xFF
    });
}

/// Retrieve the remaining frame payload length.
macro_rules! payload_length {
    ($parser:expr) => ({
        $parser.bit_data32a >> 8
    });
}

/// Parse payload data.
macro_rules! parse_payload_data {
    ($parser:expr, $handler:expr, $context:expr, $callback:ident) => ({
        if bs_available!($context) >= actual_length!($parser) as usize {
            // collect remaining data
            bs_jump!($context, actual_length!($parser) as usize);

            dec_payload_length!($parser, actual_length!($parser));

            if $parser.bit_data32a & 0xFF > 0 {
                set_state!($parser, FramePadding, frame_padding);
            } else {
                set_state!($parser, FrameLength1, frame_length1);
            }

            if $handler.$callback(bs_slice!($context), true) {
                transition!($parser, $context);
            } else {
                exit_callback!($parser, $context);
            }
        }

        // collect remaining slice
        dec_payload_length!($parser, bs_available!($context) as u32);

        bs_jump!($context, bs_available!($context));

        if $handler.$callback(bs_slice!($context), false) {
            exit_eos!($parser, $context);
        } else {
            exit_callback!($parser, $context);
        }
    });
}

/// Read a u16.
macro_rules! read_u16 {
    ($context:expr, $into:expr) => ({
        $into |= ($context.stream[$context.stream_index] as u16) << 8;
        $into |= $context.stream[$context.stream_index + 1] as u16;

        bs_jump!($context, 2);
    });
}

/// Read a u32.
macro_rules! read_u32 {
    ($context:expr, $into:expr) => ({
        $into |= ($context.stream[$context.stream_index] as u32) << 24;
        $into |= ($context.stream[$context.stream_index + 1] as u32) << 16;
        $into |= ($context.stream[$context.stream_index + 2] as u32) << 8;
        $into |= $context.stream[$context.stream_index + 3] as u32;

        bs_jump!($context, 4);
    });
}

// -------------------------------------------------------------------------------------------------

/// Parser error messages.
#[derive(Clone,Copy,PartialEq)]
pub enum ParserError {
    /// Parsing has failed.
    Dead
}

impl fmt::Debug for ParserError {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            ParserError::Dead => {
                write!(formatter, "ParserError::Dead(Parser is dead)")
            },
        }
    }
}

impl fmt::Display for ParserError {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            ParserError::Dead => {
                write!(formatter, "Parser is dead")
            },
        }
    }
}

// -------------------------------------------------------------------------------------------------

/// HTTP 2.x parser.
pub struct Parser<'a, T: HttpHandler + 'a> {
    /// Bit data that stores parser state details.
    bit_data32a: u32,

    /// Bit data that stores parser state details.
    bit_data32b: u32,

    /// Bit data that stores parser state details.
    bit_data16a: u16,

    /// Bit data that stores parser state details.
    bit_data16b: u16,

    /// Total byte count processed.
    byte_count: usize,

    /// Current state.
    state: ParserState,

    /// Current state function.
    state_function: fn(&mut Parser<'a, T>, &mut T, &mut ByteStream)
    -> Result<ParserValue, ParserError>
}

impl<'a, T: HttpHandler + 'a> Parser<'a, T> {
    /// Create a new `Parser`.
    pub fn new() -> Parser<'a, T> {
        Parser{ bit_data32a:    0,
                bit_data32b:    0,
                bit_data16a:    0,
                bit_data16b:    0,
                byte_count:     0,
                state:          ParserState::FrameLength1,
                state_function: Parser::frame_length1 }
    }

    /// Retrieve the total byte count processed since the instantiation of `Parser`.
    ///
    /// The byte count is updated when `resume()` completes. This means that if a
    /// call to `byte_count()` is executed from within a callback, it will be accurate within
    /// `stream.len()` bytes. For precise accuracy, the best time to retrieve the byte count is
    /// outside of all callbacks.
    pub fn byte_count(&self) -> usize {
        self.byte_count
    }

    /// Reset `Parser` to its initial state.
    pub fn reset(&mut self) {
        self.byte_count     = 0;
        self.state          = ParserState::FrameLength1;
        self.state_function = Parser::frame_length1;

        self.reset_bit_data();
    }

    /// Reset bit data.
    fn reset_bit_data(&mut self) {
        self.bit_data32a = 0;
        self.bit_data32b = 0;
        self.bit_data16a = 0;
        self.bit_data16b = 0;
    }

    /// Resume parsing an additional slice of data.
    ///
    /// # Arguments
    ///
    /// **`handler`**
    ///
    /// The handler implementation.
    ///
    /// **`stream`**
    ///
    /// The stream of data to be parsed.
    #[inline]
    pub fn resume(&mut self, mut handler: &mut T, stream: &[u8]) -> Result<Success, ParserError> {
        let mut context = ByteStream::new(stream);

        loop {
            match (self.state_function)(self, &mut handler, &mut context) {
                Ok(ParserValue::Continue) => {
                },
                Ok(ParserValue::Exit(success)) => {
                    self.byte_count += context.stream_index;

                    if let Success::Finished(_) = success {
                        self.state = ParserState::Finished;
                    }

                    return Ok(success);
                },
                Err(error) => {
                    self.byte_count     += context.stream_index;
                    self.state           = ParserState::Dead;
                    self.state_function  = Parser::dead;

                    return Err(error);
                }
            }
        }
    }

    /// Retrieve the current state.
    pub fn state(&self) -> ParserState {
        self.state
    }

    // ---------------------------------------------------------------------------------------------
    // FRAME STATES
    // ---------------------------------------------------------------------------------------------

    #[inline]
    fn frame_length1(&mut self, handler: &mut T, context: &mut ByteStream)
    -> Result<ParserValue, ParserError> {
        exit_if_eos!(self, context);

        self.reset_bit_data();

        if bs_available!(context) >= 4 {
            // read entire length and type
            read_u32!(context, self.bit_data32a);

            transition_fast!(
                self,
                handler,
                context,
                FrameFlags,
                frame_flags
            );
        }

        // read first length byte
        self.bit_data32a |= (get_u8!(context) as u32) << 24;

        transition_fast!(
            self,
            handler,
            context,
            FrameLength2,
            frame_length2
        );
    }

    #[inline]
    fn frame_length2(&mut self, handler: &mut T, context: &mut ByteStream)
    -> Result<ParserValue, ParserError> {
        exit_if_eos!(self, context);

        self.bit_data32a |= (get_u8!(context) as u32) << 16;

        transition_fast!(
            self,
            handler,
            context,
            FrameLength3,
            frame_length3
        );
    }

    #[inline]
    fn frame_length3(&mut self, handler: &mut T, context: &mut ByteStream)
    -> Result<ParserValue, ParserError> {
        exit_if_eos!(self, context);

        self.bit_data32a |= (get_u8!(context) as u32) << 8;

        transition_fast!(
            self,
            handler,
            context,
            FrameType,
            frame_type
        );
    }

    #[inline]
    fn frame_type(&mut self, handler: &mut T, context: &mut ByteStream)
    -> Result<ParserValue, ParserError> {
        exit_if_eos!(self, context);

        self.bit_data32a |= get_u8!(context) as u32;

        transition_fast!(
            self,
            handler,
            context,
            FrameFlags,
            frame_flags
        );
    }

    #[inline]
    fn frame_flags(&mut self, handler: &mut T, context: &mut ByteStream)
    -> Result<ParserValue, ParserError> {
        exit_if_eos!(self, context);

        self.bit_data16a = get_u8!(context) as u16;

        transition_fast!(
            self,
            handler,
            context,
            FrameStreamId1,
            frame_stream_id1
        );
    }

    #[inline]
    fn frame_stream_id1(&mut self, handler: &mut T, context: &mut ByteStream)
    -> Result<ParserValue, ParserError> {
        exit_if_eos!(self, context);

        if bs_available!(context) >= 4 {
            // read entire stream id
            read_u32!(context, self.bit_data32b);

            transition_fast!(
                self,
                handler,
                context,
                FrameFormatEnd,
                frame_format_end
            );
        }

        // read first stream id byte
        self.bit_data32b = (get_u8!(context) as u32) << 24;

        transition_fast!(
            self,
            handler,
            context,
            FrameStreamId2,
            frame_stream_id2
        );
    }

    #[inline]
    fn frame_stream_id2(&mut self, handler: &mut T, context: &mut ByteStream)
    -> Result<ParserValue, ParserError> {
        exit_if_eos!(self, context);

        self.bit_data32b = (get_u8!(context) as u32) << 16;

        transition_fast!(
            self,
            handler,
            context,
            FrameStreamId3,
            frame_stream_id3
        );
    }

    #[inline]
    fn frame_stream_id3(&mut self, handler: &mut T, context: &mut ByteStream)
    -> Result<ParserValue, ParserError> {
        exit_if_eos!(self, context);

        self.bit_data32b = (get_u8!(context) as u32) << 8;

        transition_fast!(
            self,
            handler,
            context,
            FrameStreamId4,
            frame_stream_id4
        );
    }

    #[inline]
    fn frame_stream_id4(&mut self, handler: &mut T, context: &mut ByteStream)
    -> Result<ParserValue, ParserError> {
        exit_if_eos!(self, context);

        self.bit_data32b |= get_u8!(context) as u32;

        transition_fast!(
            self,
            handler,
            context,
            FrameFormatEnd,
            frame_format_end
        );
    }

    #[inline]
    fn frame_format_end(&mut self, handler: &mut T, context: &mut ByteStream)
    -> Result<ParserValue, ParserError> {
        // match frame type
        match (self.bit_data32a & 0xFF) as u8 {
            FR_DATA => {
                if has_flag!(self, FL_PADDED) {
                    set_state!(self, DataPadLength, data_pad_length);
                } else {
                    set_state!(self, DataData, data_data);
                }
            },
            FR_HEADERS => {
                if has_flag!(self, FL_PADDED) {
                    if has_flag!(self, FL_PRIORITY) {
                        set_state!(
                            self,
                            HeadersPadLengthWithPriority,
                            headers_pad_length_with_priority
                        );
                    } else {
                        set_state!(
                            self,
                            HeadersPadLengthWithoutPriority,
                            headers_pad_length_without_priority
                        );
                    }
                } else if has_flag!(self, FL_PRIORITY) {
                    set_state!(self, HeadersStreamId1, headers_stream_id1);
                } else {
                    set_state!(self, HeadersFragment, headers_fragment);
                }
            },
            FR_PRIORITY => {
                set_state!(self, PriorityStreamId1, priority_stream_id1);
            },
            FR_RST_STREAM => {
                set_state!(self, RstStreamErrorCode1, rst_stream_error_code1);
            },
            FR_SETTINGS => {
                set_state!(self, SettingsId1, settings_id1);
            },
            FR_PUSH_PROMISE => {
                if has_flag!(self, FL_PADDED) {
                    set_state!(self, PushPromisePadLength, push_promise_pad_length);
                } else {
                    set_state!(self, PushPromiseStreamId1, push_promise_stream_id1);
                }
            },
            FR_PING => {
                set_state!(self, PingData, ping_data);
            },
            FR_GO_AWAY => {
                set_state!(self, GoAwayStreamId1, go_away_stream_id1);
            },
            FR_WINDOW_UPDATE => {
                set_state!(self, WindowUpdateIncrement1, window_update_increment1);
            },
            FR_CONTINUATION => {
                set_state!(self, HeadersFragment, headers_fragment);
            },
            _ => {
                // unsupported frame type
                if has_flag!(self, FL_PADDED) {
                    set_state!(self, UnsupportedPadLength, unsupported_pad_length);
                } else {
                    set_state!(self, UnsupportedData, unsupported_data);
                }
            }
        }

        if handler.on_frame_format(
            payload_length!(self),
            (self.bit_data32a & 0xFF) as u8,
            self.bit_data16a as u8,
            self.bit_data32b & 0x7FFFFFFF
        ) {
            self.bit_data16a  = 0;
            self.bit_data32a &= 0xFFFFFF00;
            self.bit_data32b  = 0;

            transition_fast!(self, handler, context);
        } else {
            self.bit_data16a  = 0;
            self.bit_data32a &= 0xFFFFFF00;
            self.bit_data32b  = 0;

            exit_callback!(self, context);
        }
    }

    #[inline]
    fn frame_padding(&mut self, _handler: &mut T, context: &mut ByteStream)
    -> Result<ParserValue, ParserError> {
        exit_if_eos!(self, context);

        if bs_available!(context) >= payload_length!(self) as usize {
            // consume remaining padding
            bs_jump!(context, payload_length!(self) as usize);

            transition!(
                self,
                context,
                FrameLength1,
                frame_length1
            );
        }

        // consume remaining stream
        dec_payload_length!(self, bs_available!(context) as u32);

        bs_jump!(context, bs_available!(context));

        exit_eos!(self, context);
    }

    // ---------------------------------------------------------------------------------------------
    // DATA STATES
    // ---------------------------------------------------------------------------------------------

    #[inline]
    fn data_pad_length(&mut self, handler: &mut T, context: &mut ByteStream)
    -> Result<ParserValue, ParserError> {
        exit_if_eos!(self, context);

        self.bit_data32a |= get_u8!(context) as u32;

        dec_payload_length!(self, 1);

        transition_fast!(
            self,
            handler,
            context,
            DataData,
            data_data
        );
    }

    #[inline]
    fn data_data(&mut self, handler: &mut T, context: &mut ByteStream)
    -> Result<ParserValue, ParserError> {
        exit_if_eos!(self, context);

        parse_payload_data!(
            self,
            handler,
            context,
            on_data
        );
    }

    // ---------------------------------------------------------------------------------------------
    // GO AWAY STATES
    // ---------------------------------------------------------------------------------------------

    #[inline]
    fn go_away_stream_id1(&mut self, handler: &mut T, context: &mut ByteStream)
    -> Result<ParserValue, ParserError> {
        exit_if_eos!(self, context);

        if bs_available!(context) >= 4 {
            // read reserved bit and entire stream id
            read_u32!(context, self.bit_data32b);

            transition_fast!(
                self,
                handler,
                context,
                GoAwayErrorCode1,
                go_away_error_code1
            );
        }

        // read reserved bit and first stream id byte
        self.bit_data32b |= (get_u8!(context) as u32) << 24;

        transition_fast!(
            self,
            handler,
            context,
            GoAwayStreamId2,
            go_away_stream_id2
        );
    }

    #[inline]
    fn go_away_stream_id2(&mut self, handler: &mut T, context: &mut ByteStream)
    -> Result<ParserValue, ParserError> {
        exit_if_eos!(self, context);

        self.bit_data32b |= (get_u8!(context) as u32) << 16;

        transition_fast!(
            self,
            handler,
            context,
            GoAwayStreamId3,
            go_away_stream_id3
        );
    }

    #[inline]
    fn go_away_stream_id3(&mut self, handler: &mut T, context: &mut ByteStream)
    -> Result<ParserValue, ParserError> {
        exit_if_eos!(self, context);

        self.bit_data32b |= (get_u8!(context) as u32) << 8;

        transition_fast!(
            self,
            handler,
            context,
            GoAwayStreamId3,
            go_away_stream_id3
        );
    }

    #[inline]
    fn go_away_stream_id4(&mut self, handler: &mut T, context: &mut ByteStream)
    -> Result<ParserValue, ParserError> {
        exit_if_eos!(self, context);

        self.bit_data32b |= get_u8!(context) as u32;

        transition_fast!(
            self,
            handler,
            context,
            GoAwayErrorCode1,
            go_away_error_code1
        );
    }

    #[inline]
    fn go_away_error_code1(&mut self, handler: &mut T, context: &mut ByteStream)
    -> Result<ParserValue, ParserError> {
        exit_if_eos!(self, context);

        if bs_available!(context) >= 4 {
            // read entire error code
            read_u16!(context, self.bit_data16a);
            read_u16!(context, self.bit_data16b);

            transition_fast!(
                self,
                handler,
                context,
                GoAwayCallback,
                go_away_callback
            );
        }

        // read first stream error code byte
        self.bit_data16a |= (get_u8!(context) as u16) << 8;

        transition_fast!(
            self,
            handler,
            context,
            GoAwayErrorCode2,
            go_away_error_code2
        );
    }

    #[inline]
    fn go_away_error_code2(&mut self, handler: &mut T, context: &mut ByteStream)
    -> Result<ParserValue, ParserError> {
        exit_if_eos!(self, context);

        self.bit_data16a |= get_u8!(context) as u16;

        transition_fast!(
            self,
            handler,
            context,
            GoAwayErrorCode3,
            go_away_error_code3
        );
    }

    #[inline]
    fn go_away_error_code3(&mut self, handler: &mut T, context: &mut ByteStream)
    -> Result<ParserValue, ParserError> {
        exit_if_eos!(self, context);

        self.bit_data16b |= (get_u8!(context) as u16) << 8;

        transition_fast!(
            self,
            handler,
            context,
            GoAwayErrorCode4,
            go_away_error_code4
        );
    }

    #[inline]
    fn go_away_error_code4(&mut self, handler: &mut T, context: &mut ByteStream)
    -> Result<ParserValue, ParserError> {
        exit_if_eos!(self, context);

        self.bit_data16b |= get_u8!(context) as u16;

        transition_fast!(
            self,
            handler,
            context,
            GoAwayCallback,
            go_away_callback
        );
    }

    #[inline]
    fn go_away_callback(&mut self, handler: &mut T, context: &mut ByteStream)
    -> Result<ParserValue, ParserError> {
        dec_payload_length!(self, 8);

        if payload_length!(self) > 0 {
            set_state!(self, GoAwayDebugData, go_away_debug_data);
        } else {
            set_state!(self, FrameLength1, frame_length1);
        }

        if handler.on_go_away(
            self.bit_data32b & 0x7FFFFFFF,
            ((self.bit_data16a as u32) << 16 | self.bit_data16b as u32)
        ) {
            transition_fast!(self, handler, context);
        } else {
            exit_callback!(self, context);
        }
    }

    #[inline]
    fn go_away_debug_data(&mut self, handler: &mut T, context: &mut ByteStream)
    -> Result<ParserValue, ParserError> {
        exit_if_eos!(self, context);

        parse_payload_data!(
            self,
            handler,
            context,
            on_go_away_debug_data
        );
    }

    // ---------------------------------------------------------------------------------------------
    // HEADERS STATES
    // ---------------------------------------------------------------------------------------------

    #[inline]
    fn headers_pad_length_with_priority(&mut self, handler: &mut T, context: &mut ByteStream)
    -> Result<ParserValue, ParserError> {
        exit_if_eos!(self, context);

        self.bit_data32a |= get_u8!(context) as u32;

        dec_payload_length!(self, 1);

        transition_fast!(
            self,
            handler,
            context,
            HeadersStreamId1,
            headers_stream_id1
        );
    }

    #[inline]
    fn headers_pad_length_without_priority(&mut self, handler: &mut T, context: &mut ByteStream)
    -> Result<ParserValue, ParserError> {
        exit_if_eos!(self, context);

        self.bit_data32a |= get_u8!(context) as u32;

        dec_payload_length!(self, 1);

        transition_fast!(
            self,
            handler,
            context,
            HeadersCallback,
            headers_callback
        );
    }

    #[inline]
    fn headers_stream_id1(&mut self, handler: &mut T, context: &mut ByteStream)
    -> Result<ParserValue, ParserError> {
        exit_if_eos!(self, context);

        if bs_available!(context) >= 4 {
            // read exclusive bit and entire stream id
            read_u32!(context, self.bit_data32b);

            transition_fast!(
                self,
                handler,
                context,
                HeadersWeight,
                headers_weight
            );
        }

        // read exclusive bit and first stream id byte
        self.bit_data32b |= (get_u8!(context) as u32) << 24;

        transition_fast!(
            self,
            handler,
            context,
            HeadersStreamId2,
            headers_stream_id2
        );
    }

    #[inline]
    fn headers_stream_id2(&mut self, handler: &mut T, context: &mut ByteStream)
    -> Result<ParserValue, ParserError> {
        exit_if_eos!(self, context);

        self.bit_data32b |= (get_u8!(context) as u32) << 16;

        transition_fast!(
            self,
            handler,
            context,
            HeadersStreamId3,
            headers_stream_id3
        );
    }

    #[inline]
    fn headers_stream_id3(&mut self, handler: &mut T, context: &mut ByteStream)
    -> Result<ParserValue, ParserError> {
        exit_if_eos!(self, context);

        self.bit_data32b |= (get_u8!(context) as u32) << 8;

        transition_fast!(
            self,
            handler,
            context,
            HeadersStreamId4,
            headers_stream_id4
        );
    }

    #[inline]
    fn headers_stream_id4(&mut self, handler: &mut T, context: &mut ByteStream)
    -> Result<ParserValue, ParserError> {
        exit_if_eos!(self, context);

        self.bit_data32b |= get_u8!(context) as u32;

        transition_fast!(
            self,
            handler,
            context,
            HeadersWeight,
            headers_weight
        );
    }

    #[inline]
    fn headers_weight(&mut self, handler: &mut T, context: &mut ByteStream)
    -> Result<ParserValue, ParserError> {
        exit_if_eos!(self, context);

        self.bit_data16a = (get_u8!(context) as u16) << 8;

        // decrease payload by stream id (4) and weight (1)
        dec_payload_length!(self, 5);

        transition_fast!(
            self,
            handler,
            context,
            HeadersCallback,
            headers_callback
        );
    }

    #[inline]
    fn headers_callback(&mut self, handler: &mut T, context: &mut ByteStream)
    -> Result<ParserValue, ParserError> {
        if handler.on_headers(
            self.bit_data32b >> 31 == 1,
            self.bit_data32b & 0x7FFFFFFF,
            (self.bit_data16a >> 8) as u8
        ) {
            transition_fast!(
                self,
                handler,
                context,
                HeadersFragment,
                headers_fragment
            );
        }

        exit_callback!(
            self,
            context,
            HeadersFragment,
            headers_fragment
        );
    }

    #[inline]
    fn headers_fragment(&mut self, handler: &mut T, context: &mut ByteStream)
    -> Result<ParserValue, ParserError> {
        exit_if_eos!(self, context);

        parse_payload_data!(
            self,
            handler,
            context,
            on_headers_fragment
        );
    }

    // ---------------------------------------------------------------------------------------------
    // PING STATES
    // ---------------------------------------------------------------------------------------------

    #[inline]
    fn ping_data(&mut self, handler: &mut T, context: &mut ByteStream)
    -> Result<ParserValue, ParserError> {
        exit_if_eos!(self, context);

        parse_payload_data!(
            self,
            handler,
            context,
            on_ping
        );
    }

    // ---------------------------------------------------------------------------------------------
    // PRIORITY STATES
    // ---------------------------------------------------------------------------------------------

    #[inline]
    fn priority_stream_id1(&mut self, handler: &mut T, context: &mut ByteStream)
    -> Result<ParserValue, ParserError> {
        exit_if_eos!(self, context);

        if bs_available!(context) >= 4 {
            // read exclusive bit and entire stream id
            read_u32!(context, self.bit_data32b);

            transition_fast!(
                self,
                handler,
                context,
                PriorityWeight,
                priority_weight
            );
        }

        // read exclusive bit and first stream id byte
        self.bit_data32b |= (get_u8!(context) as u32) << 24;

        transition_fast!(
            self,
            handler,
            context,
            PriorityStreamId2,
            priority_stream_id2
        );
    }

    #[inline]
    fn priority_stream_id2(&mut self, handler: &mut T, context: &mut ByteStream)
    -> Result<ParserValue, ParserError> {
        exit_if_eos!(self, context);

        self.bit_data32b |= (get_u8!(context) as u32) << 16;

        transition_fast!(
            self,
            handler,
            context,
            PriorityStreamId3,
            priority_stream_id3
        );
    }

    #[inline]
    fn priority_stream_id3(&mut self, handler: &mut T, context: &mut ByteStream)
    -> Result<ParserValue, ParserError> {
        exit_if_eos!(self, context);

        self.bit_data32b |= (get_u8!(context) as u32) << 8;

        transition_fast!(
            self,
            handler,
            context,
            PriorityStreamId4,
            priority_stream_id4
        );
    }

    #[inline]
    fn priority_stream_id4(&mut self, handler: &mut T, context: &mut ByteStream)
    -> Result<ParserValue, ParserError> {
        exit_if_eos!(self, context);

        self.bit_data32b |= get_u8!(context) as u32;

        transition_fast!(
            self,
            handler,
            context,
            PriorityWeight,
            priority_weight
        );
    }

    #[inline]
    fn priority_weight(&mut self, handler: &mut T, context: &mut ByteStream)
    -> Result<ParserValue, ParserError> {
        exit_if_eos!(self, context);

        if handler.on_priority(
            (self.bit_data32b >> 31) == 1,
            self.bit_data32b & 0x7FFFFFFF,
            get_u8!(context)
        ) {
            transition!(
                self,
                context,
                FrameLength1,
                frame_length1
            );
        }

        exit_callback!(
            self,
            context,
            FrameLength1,
            frame_length1
        );
    }

    // ---------------------------------------------------------------------------------------------
    // PUSH PROMISE STATES
    // ---------------------------------------------------------------------------------------------

    #[inline]
    fn push_promise_pad_length(&mut self, handler: &mut T, context: &mut ByteStream)
    -> Result<ParserValue, ParserError> {
        exit_if_eos!(self, context);

        self.bit_data32a |= get_u8!(context) as u32;

        dec_payload_length!(self, 1);

        transition_fast!(
            self,
            handler,
            context,
            PushPromiseStreamId1,
            push_promise_stream_id1
        );
    }

    #[inline]
    fn push_promise_stream_id1(&mut self, handler: &mut T, context: &mut ByteStream)
    -> Result<ParserValue, ParserError> {
        exit_if_eos!(self, context);

        if bs_available!(context) >= 4 {
            // read entire promised stream id
            read_u32!(context, self.bit_data32b);

            transition_fast!(
                self,
                handler,
                context,
                PushPromiseCallback,
                push_promise_callback
            );
        }

        // read first promised stream id byte
        self.bit_data32b |= (get_u8!(context) as u32) << 24;

        transition_fast!(
            self,
            handler,
            context,
            PushPromiseStreamId2,
            push_promise_stream_id2
        );
    }

    #[inline]
    fn push_promise_stream_id2(&mut self, handler: &mut T, context: &mut ByteStream)
    -> Result<ParserValue, ParserError> {
        exit_if_eos!(self, context);

        self.bit_data32b |= (get_u8!(context) as u32) << 16;

        transition_fast!(
            self,
            handler,
            context,
            PushPromiseStreamId3,
            push_promise_stream_id3
        );
    }

    #[inline]
    fn push_promise_stream_id3(&mut self, handler: &mut T, context: &mut ByteStream)
    -> Result<ParserValue, ParserError> {
        exit_if_eos!(self, context);

        self.bit_data32b |= (get_u8!(context) as u32) << 8;

        transition_fast!(
            self,
            handler,
            context,
            PushPromiseStreamId4,
            push_promise_stream_id4
        );
    }

    #[inline]
    fn push_promise_stream_id4(&mut self, handler: &mut T, context: &mut ByteStream)
    -> Result<ParserValue, ParserError> {
        exit_if_eos!(self, context);

        self.bit_data32b |= get_u8!(context) as u32;

        transition_fast!(
            self,
            handler,
            context,
            PushPromiseCallback,
            push_promise_callback
        );
    }

    #[inline]
    fn push_promise_callback(&mut self, handler: &mut T, context: &mut ByteStream)
    -> Result<ParserValue, ParserError> {
        exit_if_eos!(self, context);

        // decrease payload by stream id (4)
        dec_payload_length!(self, 4);

        if handler.on_push_promise(self.bit_data32b) {
            transition!(
                self,
                context,
                HeadersFragment,
                headers_fragment
            );
        }

        exit_callback!(
            self,
            context,
            HeadersFragment,
            headers_fragment
        );
    }

    // ---------------------------------------------------------------------------------------------
    // RST STREAM STATES
    // ---------------------------------------------------------------------------------------------

    #[inline]
    fn rst_stream_error_code1(&mut self, handler: &mut T, context: &mut ByteStream)
    -> Result<ParserValue, ParserError> {
        exit_if_eos!(self, context);

        if bs_available!(context) >= 4 {
            // read entire error code
            read_u32!(context, self.bit_data32b);

            transition_fast!(
                self,
                handler,
                context,
                RstStreamCallback,
                rst_stream_callback
            );
        }

        // read first error code byte
        self.bit_data32b |= (get_u8!(context) as u32) << 24;

        transition_fast!(
            self,
            handler,
            context,
            RstStreamErrorCode2,
            rst_stream_error_code2
        );
    }

    #[inline]
    fn rst_stream_error_code2(&mut self, handler: &mut T, context: &mut ByteStream)
    -> Result<ParserValue, ParserError> {
        exit_if_eos!(self, context);

        self.bit_data32b |= (get_u8!(context) as u32) << 16;

        transition_fast!(
            self,
            handler,
            context,
            RstStreamErrorCode3,
            rst_stream_error_code3
        );
    }

    #[inline]
    fn rst_stream_error_code3(&mut self, handler: &mut T, context: &mut ByteStream)
    -> Result<ParserValue, ParserError> {
        exit_if_eos!(self, context);

        self.bit_data32b |= (get_u8!(context) as u32) << 8;

        transition_fast!(
            self,
            handler,
            context,
            RstStreamErrorCode4,
            rst_stream_error_code4
        );
    }

    #[inline]
    fn rst_stream_error_code4(&mut self, handler: &mut T, context: &mut ByteStream)
    -> Result<ParserValue, ParserError> {
        exit_if_eos!(self, context);

        self.bit_data32b |= get_u8!(context) as u32;

        transition_fast!(
            self,
            handler,
            context,
            RstStreamCallback,
            rst_stream_callback
        );
    }

    #[inline]
    fn rst_stream_callback(&mut self, handler: &mut T, context: &mut ByteStream)
    -> Result<ParserValue, ParserError> {
        if handler.on_rst_stream(self.bit_data32b) {
            transition!(
                self,
                context,
                FrameLength1,
                frame_length1
            );
        }

        exit_callback!(
            self,
            context,
            FrameLength1,
            frame_length1
        );
    }

    // ---------------------------------------------------------------------------------------------
    // SETTINGS STATES
    // ---------------------------------------------------------------------------------------------

    #[inline]
    fn settings_id1(&mut self, handler: &mut T, context: &mut ByteStream)
    -> Result<ParserValue, ParserError> {
        exit_if_eos!(self, context);

        if bs_available!(context) >= 2 {
            // read entire identifier
            read_u16!(context, self.bit_data16a);

            transition_fast!(
                self,
                handler,
                context,
                SettingsValue1,
                settings_value1
            );
        }

        // read first identifier byte
        self.bit_data16a |= (get_u8!(context) as u16) << 8;

        transition_fast!(
            self,
            handler,
            context,
            SettingsId2,
            settings_id2
        );
    }

    #[inline]
    fn settings_id2(&mut self, handler: &mut T, context: &mut ByteStream)
    -> Result<ParserValue, ParserError> {
        exit_if_eos!(self, context);

        self.bit_data16a |= get_u8!(context) as u16;

        transition_fast!(
            self,
            handler,
            context,
            SettingsValue1,
            settings_value1
        );
    }

    #[inline]
    fn settings_value1(&mut self, handler: &mut T, context: &mut ByteStream)
    -> Result<ParserValue, ParserError> {
        exit_if_eos!(self, context);

        if bs_available!(context) >= 4 {
            // read entire value
            read_u32!(context, self.bit_data32b);

            transition_fast!(
                self,
                handler,
                context,
                SettingsCallback,
                settings_callback
            );
        }

        // read first value byte
        self.bit_data32b |= (get_u8!(context) as u32) << 24;

        transition_fast!(
            self,
            handler,
            context,
            SettingsValue2,
            settings_value2
        );
    }

    #[inline]
    fn settings_value2(&mut self, handler: &mut T, context: &mut ByteStream)
    -> Result<ParserValue, ParserError> {
        exit_if_eos!(self, context);

        self.bit_data32b |= (get_u8!(context) as u32) << 16;

        transition_fast!(
            self,
            handler,
            context,
            SettingsValue3,
            settings_value3
        );
    }

    #[inline]
    fn settings_value3(&mut self, handler: &mut T, context: &mut ByteStream)
    -> Result<ParserValue, ParserError> {
        exit_if_eos!(self, context);

        self.bit_data32b |= (get_u8!(context) as u32) << 8;

        transition_fast!(
            self,
            handler,
            context,
            SettingsValue3,
            settings_value3
        );
    }

    #[inline]
    fn settings_value4(&mut self, handler: &mut T, context: &mut ByteStream)
    -> Result<ParserValue, ParserError> {
        exit_if_eos!(self, context);

        self.bit_data32b |= get_u8!(context) as u32;

        transition_fast!(
            self,
            handler,
            context,
            SettingsCallback,
            settings_callback
        );
    }

    #[inline]
    fn settings_callback(&mut self, handler: &mut T, context: &mut ByteStream)
    -> Result<ParserValue, ParserError> {
        if handler.on_settings(self.bit_data16a, self.bit_data32b) {
            transition!(
                self,
                context,
                FrameLength1,
                frame_length1
            )
        }

        exit_callback!(
            self,
            context,
            FrameLength1,
            frame_length1
        );
    }

    // ---------------------------------------------------------------------------------------------
    // WINDOW UPDATE STATES
    // ---------------------------------------------------------------------------------------------

    #[inline]
    fn window_update_increment1(&mut self, handler: &mut T, context: &mut ByteStream)
    -> Result<ParserValue, ParserError> {
        exit_if_eos!(self, context);

        if bs_available!(context) >= 4 {
            // read entire size increment
            read_u32!(context, self.bit_data32b);

            transition_fast!(
                self,
                handler,
                context,
                WindowUpdateCallback,
                window_update_callback
            );
        }

        // read first size increment byte
        self.bit_data32b |= (get_u8!(context) as u32) << 24;

        transition_fast!(
            self,
            handler,
            context,
            WindowUpdateIncrement2,
            window_update_increment2
        );
    }

    #[inline]
    fn window_update_increment2(&mut self, handler: &mut T, context: &mut ByteStream)
    -> Result<ParserValue, ParserError> {
        exit_if_eos!(self, context);

        self.bit_data32b |= (get_u8!(context) as u32) << 16;

        transition_fast!(
            self,
            handler,
            context,
            WindowUpdateIncrement3,
            window_update_increment3
        );
    }

    #[inline]
    fn window_update_increment3(&mut self, handler: &mut T, context: &mut ByteStream)
    -> Result<ParserValue, ParserError> {
        exit_if_eos!(self, context);

        self.bit_data32b |= (get_u8!(context) as u32) << 8;

        transition_fast!(
            self,
            handler,
            context,
            WindowUpdateIncrement4,
            window_update_increment4
        );
    }

    #[inline]
    fn window_update_increment4(&mut self, handler: &mut T, context: &mut ByteStream)
    -> Result<ParserValue, ParserError> {
        exit_if_eos!(self, context);

        self.bit_data32b |= get_u8!(context) as u32;

        transition_fast!(
            self,
            handler,
            context,
            WindowUpdateCallback,
            window_update_callback
        );
    }

    #[inline]
    fn window_update_callback(&mut self, handler: &mut T, context: &mut ByteStream)
    -> Result<ParserValue, ParserError> {
        if handler.on_window_update(self.bit_data32b) {
            transition!(
                self,
                context,
                FrameLength1,
                frame_length1
            );
        }

        exit_callback!(
            self,
            context,
            FrameLength1,
            frame_length1
        );
    }

    // ---------------------------------------------------------------------------------------------
    // UNSUPPORTED STATES
    // ---------------------------------------------------------------------------------------------

    #[inline]
    fn unsupported_pad_length(&mut self, handler: &mut T, context: &mut ByteStream)
    -> Result<ParserValue, ParserError> {
        exit_if_eos!(self, context);

        self.bit_data32a |= get_u8!(context) as u32;

        dec_payload_length!(self, 1);

        transition_fast!(
            self,
            handler,
            context,
            UnsupportedData,
            unsupported_data
        );
    }

    #[inline]
    fn unsupported_data(&mut self, handler: &mut T, context: &mut ByteStream)
    -> Result<ParserValue, ParserError> {
        exit_if_eos!(self, context);

        parse_payload_data!(
            self,
            handler,
            context,
            on_unsupported
        );
    }

    // ---------------------------------------------------------------------------------------------
    // DEAD AND FINISHED STATE
    // ---------------------------------------------------------------------------------------------

    #[inline]
    fn dead(&mut self, _handler: &mut T, _context: &mut ByteStream)
    -> Result<ParserValue, ParserError> {
        exit_error!(Dead);
    }

    #[inline]
    fn finished(&mut self, _handler: &mut T, context: &mut ByteStream)
    -> Result<ParserValue, ParserError> {
        exit_finished!(self, context);
    }
}
