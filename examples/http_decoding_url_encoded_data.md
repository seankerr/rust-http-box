## Decoding URL Encoded Data

- [http_box::util::decode()](https://docs.rs/http-box/0.1.5/http_box/util/fn.decode.html)

`decode()` accepts an encoded slice of data in the form of `&[u8]`, and it returns a decoded
`String`.

```rust
extern crate http_box;

use http_box::util::decode;

fn main() {
    match decode(b"The%20quick%20brown%20fox%20jumped%20over%20the%20lazy%20dog.") {
        Ok(string) => {
            assert_eq!(
                string,
                "The quick brown fox jumped over the lazy dog."
            );
        },
        _ => panic!()
    }
}
```
