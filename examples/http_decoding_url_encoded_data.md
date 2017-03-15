## Decoding URL encoded data

[http_box::util::decode()](https://docs.rs/http-box/0.1.3/http_box/util/fn.decode.html)

`decode()` accepts two arguments, the first being the bytes that will be
decoded, and the second being a mutable closure that will receive the decoded
segments of data.

```rust
extern crate http_box;

use http_box::util::decode;

fn main() {
    let mut v1 = Vec::new();

    decode(
        b"The%20quick%20brown%20fox%20jumped%20over%20the%20lazy%20dog.",
        |bytes| {
            v1.extend_from_slice(bytes);
        }
    );

    let mut v2 = Vec::new();

    v2.extend_from_slice(b"The quick brown fox jumped over the lazy dog.");

    assert_eq!(
        &v1,
        &v2
    );
}
```

[http_box::util::decode_into_vec()](https://docs.rs/http-box/0.1.3/http_box/util/fn.decode_into_vec.html)

`decode()` also has a shadow function `decode_into_vec()` which is a wrapper
function to save time when you're decoding into a vector.

```rust
extern crate http_box;

use http_box::util::decode_into_vec;

fn main() {
    let mut v1 = Vec::new();

    decode_into_vec(
        b"The%20quick%20brown%20fox%20jumped%20over%20the%20lazy%20dog.",
        &mut v1
    );

    let mut v2 = Vec::new();

    v2.extend_from_slice(b"The quick brown fox jumped over the lazy dog.");

    assert_eq!(
        &v1,
        &v2
    );
}
```
