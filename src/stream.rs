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
//!
//! All macros work with a base struct referred to as `$context`, and this struct must contain at
//! least these four fields:
//!
//! - `byte` (u8) The most recent byte.
//! - `mark_index` (usize) Starting index of a collection of marked bytes.
//! - `stream` (&[u8]) Stream of bytes.
//! - `stream_index` (usize) Current stream index.

use std::fmt;

/// Iterate the stream, and for each new byte execute `$exec`. If end-of-stream is located, execute
/// `$eos`.
#[macro_export]
macro_rules! stream_collect {
    ($context:expr, $eos:expr, $exec:expr) => ({
        loop {
            if stream_is_eos!($context) {
                $eos
            } else {
                stream_next!($context);

                $exec
            }
        }
    });
}

/// Collect and convert all digit bytes into an integer variable.
///
/// Exit the collection loop upon finding a non-digit byte. Return `$error` if `$digit` exceeds
/// `$max`.
#[macro_export]
macro_rules! stream_collect_digits {
    ($context:expr, $error:expr, $digit:expr, $max:expr, $eos:expr) => ({
        stream_collect!($context, $eos,
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

/// Collect `$length` bytes as long as `$allow` yields `true`. Otherwise return `$error`.
///
/// This macro assumes that `$length` bytes are available for reading.
///
/// End-of-stream returns `$error`.
#[macro_export]
macro_rules! stream_collect_length {
    ($context:expr, $error:expr, $length:expr, $allow:expr) => ({
        stream_collect!($context, {
            return Err($error($context.byte));
        }, {
            if $allow {
                if $context.stream_index == $context.mark_index + $length {
                    break;
                }
            } else {
                return Err($error($context.byte));
            }
        });
    });
}

/// Collect only `$collect`.
///
/// Exit the collection loop when `$stop` yields `true`.
#[macro_export]
macro_rules! stream_collect_only {
    ($context:expr, $error:expr, $eos:expr, $stop:expr, $collect:expr) => ({
        stream_collect!($context, $eos,
            if $stop {
                break;
            } else if !($collect) {
                return Err($error($context.byte));
            }
        );
    });
}

/// Collect all token bytes.
///
/// Exit the collection loop when `$stop` yields `true`.
#[macro_export]
macro_rules! stream_collect_tokens {
    ($context:expr, $error:expr, $eos:expr, $stop:expr) => ({
        stream_collect!($context, $eos,
            if $stop {
                break;
            } else if !is_token($context.byte) {
                return Err($error($context.byte));
            }
        );
    });

    ($context:expr, $error:expr, $eos:expr) => ({
        stream_collect!($context, $eos,
            if !is_token($context.byte) {
                return Err($error($context.byte));
            }
        );
    });
}

/// Collect all visible 7-bit bytes. Visible bytes are 0x21 thru 0x7E.
///
/// Exit the collection loop when `$stop` yields `true`.
#[macro_export]
macro_rules! stream_collect_visible {
    ($context:expr, $error:expr, $eos:expr, $stop:expr) => ({
        stream_collect!($context, $eos,
            if $stop {
                break;
            } else if is_non_visible!($context.byte) {
                return Err($error($context.byte));
            }
        );
    });

    ($context:expr, $error:expr, $eos:expr) => ({
        stream_collect!($context, $eos,
            if is_non_visible!($context.byte) {
                return Err($error($context.byte));
            }
        );
    });
}

/// Retrieve the slice of marked bytes.
macro_rules! stream_collected_bytes {
    ($context:expr) => (
        &$context.stream[$context.mark_index..$context.stream_index];
    );
}

/// Retrieve slice of marked bytes ignoring the very last byte.
macro_rules! stream_collected_bytes_ignore {
    ($context:expr) => (
        &$context.stream[$context.mark_index..$context.stream_index - 1];
    );
}

/// Find a byte within a stream.
///
/// The `$start` index is relative to `$context.stream_index`.
macro_rules! stream_find {
    ($context:expr, $start:expr, $byte:expr) => ({
        let mut index = None;

        if $context.stream_index + $start < $context.stream.len() {
            for s in $context.stream_index + $start..$context.stream.len() {
                if $byte == $context.stream[s] {
                    index = Some(s);

                    break;
                }
            }
        }

        index
    });

    ($context:expr, $byte:expr) => (
        stream_find!($context, 0, $byte);
    );
}

/// Find a pattern within a stream.
///
/// The `$start` index is relative to `$context.stream_index`.
macro_rules! stream_find_pattern {
    ($context:expr, $start:expr, $pattern:expr) => ({
        let mut index = None;

        if $context.stream_index + $start < $context.stream.len() {
            'outer:
            for s in $context.stream_index + $start..$context.stream.len() {
                for p in 0..$pattern.len() {
                    if $context.stream.len() <= s + p || $pattern[p] != $context.stream[s + p] {
                        break;
                    } else if $pattern.len() == p + 1 {
                        index = Some(s);

                        break 'outer;
                    }
                }
            }
        }

        index
    });

    ($context:expr, $pattern:expr) => (
        stream_find_pattern!($context, 0, $pattern);
    );
}

/// Indicates that a specified amount of bytes are available for reading.
macro_rules! stream_has_bytes {
    ($context:expr, $length:expr) => (
        $context.stream_index + $length <= $context.stream.len()
    );
}

/// Indicates that we're at the end of the stream.
macro_rules! stream_is_eos {
    ($context:expr) => (
        $context.stream_index == $context.stream.len()
    );
}

/// Jump `$length` bytes.
///
/// This macro assumes that `$length` bytes are available for reading.
macro_rules! stream_jump {
    ($context:expr, $length:expr) => ({
        $context.stream_index += $length;
    });
}

/// Set `$context.mark_index` to `$context.stream_index` or `$index`.
macro_rules! stream_mark {
    ($context:expr) => ({
        $context.mark_index = $context.stream_index;
    });

    ($context:expr, $index:expr) => ({
        $context.mark_index = $index;
    });
}

/// Advance `$context.stream_index` one byte and set `$context.byte` to the new byte.
macro_rules! stream_next {
    ($context:expr) => ({
        $context.byte          = $context.stream[$context.stream_index];
        $context.stream_index += 1;
    });
}

/// Peek at a slice of bytes.
///
/// This macro assumes that `$length` bytes are available for reading.
macro_rules! stream_peek {
    ($context:expr, $length:expr) => (
        &$context.stream[$context.stream_index..$context.stream_index + $length]
    );
}

/// Replay the most recent byte.
macro_rules! stream_replay {
    ($context:expr) => ({
        stream_rewind!($context, 1);
    });
}

/// Rewind `$context.stream_index` by `$length` bytes.
macro_rules! stream_rewind {
    ($context:expr, $length:expr) => ({
        $context.stream_index -= $length;
    });
}

/// Rewind `$context.stream_index` to index `$index`.
macro_rules! stream_rewind_to {
    ($context:expr, $index:expr) => ({
        $context.stream_index = $index;
    });
}

// -------------------------------------------------------------------------------------------------

/// Default stream context type.
pub struct Context<'a> {
    // Current byte.
    pub byte: u8,

    // Callback mark index.
    pub mark_index: usize,

    // Stream data.
    pub stream: &'a [u8],

    // Stream index.
    pub stream_index: usize
}

impl<'a> Context<'a> {
    /// Create a new `Context`.
    pub fn new(stream: &'a [u8]) -> Context<'a> {
        Context{ byte:         0,
                 mark_index:   0,
                 stream:       stream,
                 stream_index: 0 }
    }
}

impl<'a> fmt::Debug for Context<'a> {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "Context(byte[{}]='{}', mark_index={}, stream_index={})",
               self.byte, self.byte as char, self.mark_index, self.stream_index)
    }
}

impl<'a> fmt::Display for Context<'a> {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        if self.byte < 0x20 || self.byte == 0x7F {
            write!(formatter, "byte[{}]='', mark_index={}, stream_index={}",
                   self.byte, self.mark_index, self.stream_index)
        } else {
            write!(formatter, "byte[{}]='{}', mark_index={}, stream_index={}",
                   self.byte, self.byte as char, self.mark_index, self.stream_index)
        }
    }
}
