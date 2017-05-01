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

//! Stream collection macros.

/// Collect an unquoted header field value.
///
/// Exit the collection loop upon finding an invalid byte, or when `$stop` is `true`.
macro_rules! collect_field {
    ($context:expr, $stop:expr, $on_eos:expr) => ({
        bs_collect!($context,
            if !is_header_field($context.byte) || $stop {
                break;
            },
            $on_eos
        );
    });

    ($context:expr, $on_eos:expr) => ({
        bs_collect!($context,
            if !is_header_field($context.byte) {
                break;
            },
            $on_eos
        );
    });
}

/// Collect and convert 2 hex bytes into a u8.
///
/// This macro assumes that 2 bytes are available for reading. Return `$error` upon locating a
/// non-hex byte.
macro_rules! collect_hex8 {
    ($context:expr, $error:expr) => ({
        bs_next!($context);

        (
            if is_digit!($context.byte) {
                ($context.byte - b'0') << 4
            } else if b'@' < $context.byte && $context.byte < b'G' {
                ($context.byte - 0x37) << 4
            } else if b'`' < $context.byte && $context.byte < b'g' {
                ($context.byte - 0x57) << 4
            } else {
                return Err($error($context.byte));
            } as u8
        )
        +
        {
            bs_next!($context);

            (
                if is_digit!($context.byte) {
                    $context.byte - b'0'
                } else if b'@' < $context.byte && $context.byte < b'G' {
                    $context.byte - 0x37
                } else if b'`' < $context.byte && $context.byte < b'g' {
                    $context.byte - 0x57
                } else {
                    return Err($error($context.byte));
                } as u8
            )
        }
    });
}

/// Collect and convert 2 hex bytes into a u8 variable.
///
/// This macro is compatible with custom iterators.
///
/// This macro assumes that 2 bytes are available for reading. Return `$error` upon locating a
/// non-hex byte.
macro_rules! collect_hex8_iter {
    ($iter:expr, $context:expr, $error:expr) => ({
        bs_next!($context);

        (
            if is_digit!($context.byte) {
                ($context.byte - b'0') << 4
            } else if b'@' < $context.byte && $context.byte < b'G' {
                ($context.byte - 0x37) << 4
            } else if b'`' < $context.byte && $context.byte < b'g' {
                ($context.byte - 0x57) << 4
            } else {
                (*$iter.on_error)($error($context.byte));

                return None;
            } as u8
        )
        +
        {
            bs_next!($context);

            (
                if is_digit!($context.byte) {
                    $context.byte - b'0'
                } else if b'@' < $context.byte && $context.byte < b'G' {
                    $context.byte - 0x37
                } else if b'`' < $context.byte && $context.byte < b'g' {
                    $context.byte - 0x57
                } else {
                    (*$iter.on_error)($error($context.byte));

                    return None;
                } as u8
            )
        }
    });
}

/// Collect a quoted header field value.
///
/// Exit the collection loop upon finding an invalid byte.
macro_rules! collect_quoted_field {
    ($context:expr, $on_eos:expr) => ({
        bs_collect!($context,
            if !is_quoted_header_field($context.byte) {
                break;
            },
            $on_eos
        );
    });
}

/// Collect all token bytes.
///
/// Exit the collection loop when `$stop` yields `true`.
macro_rules! collect_tokens {
    ($context:expr, $stop:expr, $on_eos:expr) => ({
        bs_collect!($context,
            if !is_token($context.byte) || $stop {
                break;
            },
            $on_eos
        );
    });

    ($context:expr, $on_eos:expr) => ({
        bs_collect!($context,
            if !is_token($context.byte) {
                break;
            },
            $on_eos
        );
    });
}

/// Collect all visible 7-bit bytes. Visible bytes are 0x21 thru 0x7E.
///
/// Exit the collection loop when `$stop` yields `true`.
macro_rules! collect_visible_7bit {
    ($context:expr, $stop:expr, $on_eos:expr) => ({
        bs_collect!($context,
            if is_not_visible_7bit!($context.byte) || $stop {
                break;
            },
            $on_eos
        );
    });

    ($context:expr, $on_eos:expr) => ({
        bs_collect!($context,
            if is_not_visible_7bit!($context.byte) {
                break;
            },
            $on_eos
        );
    });
}

/// Consume all linear white space bytes.
///
/// Exit the collection loop when a non-linear white space byte is found.
macro_rules! consume_linear_space {
    ($context:expr, $on_eos:expr) => ({
        bs_available!($context) > 0 || $on_eos;

        #[allow(unused_unsafe)]
        unsafe {
            if bs_starts_with1!($context, b" ") || bs_starts_with1!($context, b"\t") {
                loop {
                    bs_available!($context) > 0 || $on_eos;

                    bs_next!($context);

                    if !($context.byte == b' ' || $context.byte == b'\t') {
                        break;
                    }
                }
            } else {
                bs_next!($context);
            }
        }
    });
}

/// Consume all space bytes.
///
/// Exit the collection loop when a non-space byte is found.
macro_rules! consume_spaces {
    ($context:expr, $on_eos:expr) => ({
        bs_available!($context) > 0 || $on_eos;

        unsafe {
            if bs_starts_with1!($context, b" ") {
                loop {
                    bs_available!($context) > 0 || $on_eos;

                    bs_next!($context);

                    if $context.byte != b' ' {
                        break;
                    }
                }
            } else {
                bs_next!($context);
            }
        }
    });
}
