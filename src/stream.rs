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

//! Stream collection macros.

/// Collect base macro.
#[macro_export]
macro_rules! stream_collect {
    ($context:expr, $eof:expr, $stop:expr) => ({
        loop {
            if stream_is_eof!($context) {
                $eof
            }

            stream_next!($context);

            $stop
        }
    });
}

/// Collect all digit characters into an integer variable.
#[macro_export]
macro_rules! stream_collect_digits {
    ($context:expr, $error:expr, $digit:expr, $max:expr, $eof:expr) => ({
        stream_collect!($context, $eof,
            if is_digit!($context.byte) {
                $digit *= 10;
                $digit += ($context.byte - b'0') as u64;

                if $digit > $max {
                    return Err($error($context.byte));
                }
            } else {
                break;
            }
        );
    });
}

/// Collect all token characters.
///
/// Allow a pre-byte check that stops collection and returns the current byte.
#[macro_export]
macro_rules! stream_collect_tokens {
    ($context:expr, $error:expr, $eof:expr, $stop:expr) => ({
        stream_collect!($context, $eof,
            if $stop {
                break;
            } else if !is_token($context.byte) {
                return Err($error($context.byte));
            }
        );
    });

    ($context:expr, $error:expr, $eof:expr) => ({
        stream_collect!($context, $eof,
            if !is_token($context.byte) {
                return Err($error($context.byte));
            }
        );
    });
}

/// Collect all visible characters.
///
/// Allow a pre-byte check that stops collection and returns the current byte.
#[macro_export]
macro_rules! stream_collect_visible {
    ($context:expr, $error:expr, $eof:expr, $stop:expr) => ({
        stream_collect!($context, $eof,
            if $stop {
                break;
            } else if is_non_visible!($context.byte) {
                return Err($error($context.byte));
            }
        );
    });

    ($context:expr, $error:expr, $eof:expr) => ({
        stream_collect!($context, $eof,
            if is_non_visible!($context.byte) {
                return Err($error($context.byte));
            }
        );
    });
}

/// Indicates that a specified amount of bytes are available.
macro_rules! stream_has_bytes {
    ($context:expr, $length:expr) => (
        $context.stream_index + $length <= $context.stream.len()
    );
}

/// Indicates that we're at the end of the stream.
macro_rules! stream_is_eof {
    ($context:expr) => (
        $context.stream_index == $context.stream.len()
    );
}

/// Jump a specified amount of bytes.
macro_rules! stream_jump_bytes {
    ($context:expr, $length:expr) => ({
        $context.stream_index += $length;
    });
}

/// Advance the stream one byte and record the new byte.
macro_rules! stream_next {
    ($context:expr) => ({
        $context.stream_index += 1;
        $context.byte          = $context.stream[$context.stream_index - 1];
    });
}

/// Peek at a slice of available bytes.
macro_rules! stream_peek_bytes {
    ($context:expr, $length:expr) => (
        &$context.stream[$context.stream_index..$context.stream_index + $length]
    );
}

/// Replay the most recent byte by rewinding the stream index 1 byte.
macro_rules! stream_replay {
    ($context:expr) => ({
        $context.stream_index -= 1;
    });
}
