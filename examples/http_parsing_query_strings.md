## Parsing Query Strings

- [http_box::util::QueryIterator()](https://docs.rs/http-box/0.1.3/http_box/util/struct.QueryIterator.html)
- [http_box::util::QueryError](https://docs.rs/http-box/0.1.3/http_box/util/enum.QueryError.html)

`QueryIterator` enables you to iterate over a query string. Each iteration will return
`(String, Option<String>)`. You can optionally set a callback for receiving errors with
`QueryIterator::on_error()`. This callback will receive instances of `QueryError`.

Here's a basic example that ignores errors:

```rust
extern crate http_box;

use http_box::util::QueryIterator;
use std::collections::HashMap;

fn main() {
    let mut map = HashMap::new();

    let query = b"field1=value1&field2=value2&field3";

    for (name, value) in QueryIterator::new(query) {
        // ignore parameters without a value
        if value.is_some() {
            map.insert(name, value.unwrap());
        }
    }

    assert_eq!(
        map.get("field1").unwrap(),
        "value1"
    );

    assert_eq!(
        map.get("field2").unwrap(),
        "value2"
    );

    assert!(!map.contains_key("field3"));
}
```

And here's an example of specifying an error callback to handle a decoding error:

```rust
extern crate http_box;

use http_box::util::{ QueryError, QueryIterator };
use std::collections::HashMap;

fn main() {
    let mut error = None;
    let mut map   = HashMap::new();

    // notice the null byte at the end of the last parameter name
    // this will report a QueryError::Name error with the byte value that triggered the error
    let query = b"field1=value1&field2=value2&field3\0";

    for (name, value) in QueryIterator::new(query).on_error(
        |x| {
            error = Some(x);
        }
    ) {
        if value.is_some() {
            map.insert(name, value.unwrap());
        }
    }

    assert_eq!(
        map.get("field1").unwrap(),
        "value1"
    );

    assert_eq!(
        map.get("field2").unwrap(),
        "value2"
    );

    assert!(!map.contains_key("field3"));

    match error.unwrap() {
        QueryError::Name(x) => assert_eq!(x, 0),
        QueryError::Value(_) => panic!()
    }
}
```
