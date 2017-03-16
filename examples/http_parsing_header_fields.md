## Parsing Header Fields

- [http_box::util::FieldIterator()](https://docs.rs/http-box/0.1.4/http_box/util/struct.FieldIterator.html)
- [http_box::util::FieldError](https://docs.rs/http-box/0.1.4/http_box/util/enum.FieldError.html)

`FieldIterator` enables you to iterate over a header field. Each iteration will return
`(String, Option<String>)`. You can optionally set a callback for receiving errors with
`FieldIterator::on_error()`. This callback will receive instances of `FieldError`.

Here's a basic example that ignores errors:

```rust
extern crate http_box;

use http_box::util::FieldIterator;

fn main() {
    let field = b"form/multipart; boundary=\"randomlongboundary\"";

    for (n, (name, value)) in FieldIterator::new(field, b';', true).enumerate() {
        if n == 0 {
            assert_eq!(
                name,
                "form/multipart"
            );

            assert_eq!(
                value,
                None
            );
        } else if n == 1 {
            assert_eq!(
                name,
                "boundary"
            );

            assert_eq!(
                value.unwrap(),
                "randomlongboundary"
            );
        }
    }
}
```

And here's an example of specifying an error callback to handle an error:

```rust
extern crate http_box;

use http_box::util::{ FieldError, FieldIterator };

fn main() {
    // notice the missing double-quote at the end of the last value
    // this will report a FieldError::Value error with the byte value that triggered the error
    let field = b"form/multipart; boundary=\"randomlongboundary";

    for (n, (name, value)) in FieldIterator::new(field, b';', true)
    .on_error(
        |error| {
            // because the last byte was `y`, it's reported as the error byte
            match error {
                FieldError::Name(_) => panic!(),
                FieldError::Value(x) => assert_eq!(x, b'y')
            }
        }
    )
    .enumerate() {
        if n == 0 {
            assert_eq!(
                name,
                "form/multipart"
            );

            assert_eq!(
                value,
                None
            );
        }
    }
}
```
