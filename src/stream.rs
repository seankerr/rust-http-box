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

//! Macros for parsing stream data.

/// Execute a callback and if it returns true, execute a block, otherwise exit with callback status.
macro_rules! callback {
    ($parser:expr, $context:expr, $function:ident, $block:block) => ({
        let slice = collected_bytes!($parser, $context);

        if slice.len() > 0 {
            if $context.handler.$function(slice) {
                $block
            } else {
                exit_callback!($parser, $context);
            }
        } else {
            $block
        }
    });
}

/// Execute a callback with specified data, and if it returns true, execute a block, otherwise exit
/// with callback status.
macro_rules! callback_data {
    ($parser:expr, $context:expr, $data:expr, $function:ident, $block:block) => ({
        if $context.handler.$function($data) {
            $block
        } else {
            exit_callback!($parser, $context);
        }
    });
}

/// Execute a callback ignoring the last marked byte, and if it returns true, execute a block,
/// otherwise exit with callback status.
macro_rules! callback_ignore {
    ($parser:expr, $context:expr, $function:ident, $block:block) => ({
        let slice = &$context.stream[$context.mark_index..$context.stream_index - 1];

        if slice.len() > 0 {
            if $context.handler.$function(slice) {
                $block
            } else {
                exit_callback!($parser, $context);
            }
        } else {
            $block
        }
    });
}

/// Execute a callback and if it returns true, exit with eof status, otherwise exit with callback
/// status.
macro_rules! callback_or_eof {
    ($parser:expr, $context:expr, $function:ident) => (
        callback!($parser, $context, $function, {
            exit_eof!($parser, $context);
        });
    );
}

/// Change parser state.
macro_rules! change_state {
    ($parser:expr, $context:expr) => ({
        $context.mark_index = $context.stream_index;

        return Ok(ParserValue::Continue);
    });
}

/// Change parser state fast, without returning immediately.
macro_rules! change_state_fast {
    ($parser:expr, $context:expr) => ({
        $context.mark_index = $context.stream_index;

        return ($parser.state_function)($parser, $context);
    });
}

/// Retrieve a slice of collected bytes.
macro_rules! collected_bytes {
    ($parser:expr, $context:expr) => (
        &$context.stream[$context.mark_index..$context.stream_index]
    );
}

/// Retrieve a slice of collected bytes forgetting the most recent byte.
macro_rules! collected_bytes_forget {
    ($parser:expr, $context:expr) => (
        &$context.stream[$context.mark_index..$context.stream_index - 1]
    );
}

/// Collect digits.
macro_rules! collect_digits {
    ($parser:expr, $context:expr, $digit:expr, $max:expr, $error:expr, $error_msg:expr,
     $eof_block:block) => ({
        loop {
            if is_eof!($context) {
                $eof_block
            }

            next!($context);

            if is_digit!($context.byte) {
                $digit *= 10;
                $digit += ($context.byte - b'0') as u16;

                if $digit > $max {
                    exit_error!($parser, $context, $error($error_msg, $context.byte));
                }
            } else {
                break;
            }
        }
    });
}

/// Collect all 7-bit non-control bytes.
macro_rules! collect_safe {
    ($parser:expr, $context:expr, $stop1:expr, $stop2:expr, $error:expr, $error_msg:expr,
     $eof_block:block) => ({
        loop {
            if is_eof!($context) {
                $eof_block
            }

            next!($context);

            if $stop1 == $context.byte || $stop2 == $context.byte {
                break;
            } else if is_control!($context.byte) || !is_ascii!($context.byte) {
                exit_error!($parser, $context, $error($error_msg, $context.byte));
            }
        }
    });

    ($parser:expr, $context:expr, $stop:expr, $error:expr, $error_msg:expr,
     $eof_block:block) => ({
        loop {
            if is_eof!($context) {
                $eof_block
            }

            next!($context);

            if $stop == $context.byte {
                break;
            } else if is_control!($context.byte) || !is_ascii!($context.byte) {
                exit_error!($parser, $context, $error($error_msg, $context.byte));
            }
        }
    });
}

/// Collect tokens.
macro_rules! collect_tokens {
    ($parser:expr, $context:expr, $stop:expr, $error:expr, $error_msg:expr,
     $eof_block:block) => ({
        loop {
            if is_eof!($context) {
                $eof_block
            }

            next!($context);

            if $stop == $context.byte {
                break;
            } else if !is_token($context.byte) {
                exit_error!($parser, $context, $error($error_msg, $context.byte));
            }
        }
    });
}

/// Consume spaces and tabs.
macro_rules! consume_space_tab {
    ($parser:expr, $context:expr) => ({
        loop {
            if is_eof!($context) {
                exit_eof!($parser, $context);
            }

            next!($context);

            if $context.byte != b' ' && $context.byte != b'\t' {
                break;
            }
        }
    });
}

/// Exit parser function with a callback status.
macro_rules! exit_callback {
    ($parser:expr, $context:expr) => ({
        $parser.byte_count += $context.stream_index;

        return Ok(ParserValue::Exit(Success::Callback($context.stream_index)));
    });
}

/// Exit parser function with an EOF status.
macro_rules! exit_eof {
    ($parser:expr, $context:expr) => ({
        $parser.byte_count += $context.stream_index;

        return Ok(ParserValue::Exit(Success::Eof($context.stream_index)));
    });
}

/// Exit parser function with an error.
macro_rules! exit_error {
    ($parser:expr, $context:expr, $error:expr) => ({
        $parser.byte_count += $context.stream_index;
        $parser.state       = State::Dead;

        return Err($error);
    });
}

/// Exit parser with finished status.
macro_rules! exit_finished {
    ($parser:expr, $context:expr) => ({
        $parser.byte_count += $context.stream_index;
        $parser.state       = State::Finished;

        return Ok(ParserValue::Exit(Success::Finished($context.stream_index)));
    });
}

/// Exit parser function with an EOF status if the stream is EOF, otherwise do nothing.
macro_rules! exit_if_eof {
    ($parser:expr, $context:expr) => (
        if is_eof!($context) {
            exit_eof!($parser, $context);
        }
    );
}

/// Indicates that a specified amount of bytes are available.
macro_rules! has_bytes {
    ($context:expr, $length:expr) => (
        $context.stream_index + $length <= $context.stream.len()
    );
}

/// Indicates that we're at the end of the stream.
macro_rules! is_eof {
    ($context:expr) => (
        $context.stream_index == $context.stream.len()
    );
}

/// Jump a specified amount of bytes.
macro_rules! jump_bytes {
    ($context:expr, $length:expr) => ({
        $context.stream_index += $length;
    });
}

/// Advance the stream one byte.
macro_rules! next {
    ($context:expr) => ({
        $context.stream_index += 1;
        $context.byte   = $context.stream[$context.stream_index - 1]
    });
}

/// Peek at a slice of available bytes.
macro_rules! peek_bytes {
    ($context:expr, $length:expr) => (
        &$context.stream[$context.stream_index..$context.stream_index + $length]
    );
}

/// Replay the most recent byte by rewinding the stream index 1 byte.
macro_rules! replay {
    ($context:expr) => (
        $context.stream_index -= 1;
    );
}

/// Set state and state function.
macro_rules! set_state {
    ($parser:expr, $state:expr, $state_function:ident) => ({
        $parser.state          = $state;
        $parser.state_function = Parser::$state_function;
    });
}
