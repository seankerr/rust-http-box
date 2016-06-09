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

/// Collect and convert all digit bytes into a u16 variable.
///
/// Exit the collection loop upon finding a non-digit byte. Return `$error` if `$digit` exceeds
/// `$max`.
macro_rules! collect_digits16 {
    ($context:expr, $error:expr, $digit:expr, $max:expr, $on_eos:expr) => ({
        bs_collect_digits16!($context, $digit,
            if $digit > $max {
                return Err($error($context.byte));
            },
            $on_eos
        );
    });
}

/// Collect and convert all digit bytes into a u32 variable.
///
/// Exit the collection loop upon finding a non-digit byte. Return `$error` if `$digit` exceeds
/// `$max`.
macro_rules! collect_digits32 {
    ($context:expr, $error:expr, $digit:expr, $max:expr, $on_eos:expr) => ({
        bs_collect_digits32!($context, $digit,
            if $digit > $max {
                return Err($error($context.byte));
            },
            $on_eos
        );
    });
}

/// Collect remaining bytes until content length is zero.
///
/// Content length is stored in the upper 40 bits.
macro_rules! collect_content_length {
    ($parser:expr, $context:expr) => ({
        exit_if_eos!($parser, $context);

        if bs_has_bytes!($context, get_all28!($parser) as usize) {
            $context.stream_index += get_all28!($parser) as usize;

            set_all28!($parser, 0);

            true
        } else {
            $context.stream_index += $context.stream.len();

            set_all28!($parser, get_all28!($parser) as usize - $context.stream.len());

            false
        }
    });
}

/// Collect all bytes that are allowed within a quoted value.
///
/// Exit the collection loop upon finding an unescaped double quote. Return `$error` upon finding a
/// non-visible 7-bit byte that also isn't a space.
macro_rules! collect_quoted_value {
    ($parser:expr, $context:expr, $error:expr, $function:ident) => ({
        bs_collect!($context,
            if b'"' == $context.byte || b'\\' == $context.byte {
                break;
            } else if is_not_visible_7bit!($context.byte) && $context.byte != b' ' {
                return Err($error($context.byte));
            },
            callback_eos_expr!($parser, $context, $function)
        );
    });
}

/// Collect all token bytes.
///
/// Exit the collection loop when `$stop` yields `true`.
macro_rules! collect_tokens {
    ($context:expr, $error:expr, $stop:expr, $on_eos:expr) => ({
        bs_collect!($context,
            if $stop {
                break;
            } else if !is_token($context.byte) {
                return Err($error($context.byte));
            },
            $on_eos
        );
    });

    ($context:expr, $error:expr, $on_eos:expr) => ({
        bs_collect!($context,
            if !is_token($context.byte) {
                return Err($error($context.byte));
            },
            $on_eos
        );
    });
}

/// Collect all visible 7-bit bytes. Visible bytes are 0x21 thru 0x7E.
///
/// Exit the collection loop when `$stop` yields `true`.
macro_rules! collect_visible {
    ($context:expr, $error:expr, $stop:expr, $on_eos:expr) => ({
        bs_collect!($context,
            if $stop {
                break;
            } else if is_not_visible_7bit!($context.byte) {
                return Err($error($context.byte));
            },
            $on_eos
        );
    });

    ($context:expr, $error:expr, $on_eos:expr) => ({
        bs_collect!($context,
            if is_not_visible_7bit!($context.byte) {
                return Err($error($context.byte));
            },
            $on_eos
        );
    });
}

/// Consume all linear white space bytes.
///
/// Exit the collection loop when a non-linear white space byte is found.
macro_rules! consume_linear_space {
    ($parser:expr, $context:expr) => ({
        if bs_is_eos!($context) {
            exit_eos!($parser, $context);
        }

        if bs_starts_with1!($context, b" ") || bs_starts_with1!($context, b"\t") {
            loop {
                if bs_is_eos!($context) {
                    exit_eos!($parser, $context);
                }

                bs_next!($context);

                if $context.byte == b' ' || $context.byte == b'\t' {
                } else {
                    bs_replay!($context);

                    break;
                }
            }
        }
    });
}
