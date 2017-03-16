## Parsing Query Strings

- [http_box::util::QueryIterator()](https://docs.rs/http-box/0.1.5/http_box/util/struct.QueryIterator.html)
- [http_box::util::QueryError](https://docs.rs/http-box/0.1.5/http_box/util/enum.QueryError.html)

`QueryIterator` enables you to iterate over a query string. Each iteration will return
`(String, Option<String>)`. You can optionally set a callback for receiving errors with
`QueryIterator::on_error()`. This callback will receive instances of `QueryError`.

Here's a basic example that ignores errors:

```rust
extern crate http_box;

use http_box::util::QueryIterator;

fn main() {
    let query = b"field1=value1&field2=value2&field3";

    for (n, (name, value)) in QueryIterator::new(query).enumerate() {
        if n == 0 {
            assert_eq!(
                name,
                "field1"
            );

            assert_eq!(
                value.unwrap(),
                "value1"
            );
        } else if n == 1 {
            assert_eq!(
                name,
                "field2"
            );

            assert_eq!(
                value.unwrap(),
                "value2"
            );
        } else if n == 2 {
            assert_eq!(
                name,
                "field3"
            );

            assert_eq!(
                value,
                None
            );
        }
    }
}
```

And here's an example of specifying an error callback to handle a decoding error:

```rust
extern crate http_box;

use http_box::util::{ QueryError, QueryIterator };

fn main() {
    // notice the null byte at the end of the last parameter name
    // this will report a QueryError::Name error with the byte value that triggered the error
    let query = b"field1=value1&field2=value2&field3\0";

    for (n, (name, value)) in QueryIterator::new(query)
    .on_error(
        |error| {
            match error {
                QueryError::Name(x) => assert_eq!(x, 0),
                QueryError::Value(_) => panic!()
            }
        }
    )
    .enumerate() {
        if n == 0 {
            assert_eq!(
                name,
                "field1"
            );

            assert_eq!(
                value.unwrap(),
                "value1"
            );
        } else if n == 1 {
            assert_eq!(
                name,
                "field2"
            );

            assert_eq!(
                value.unwrap(),
                "value2"
            );
        }
    }
}
```
