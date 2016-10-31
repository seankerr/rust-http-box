# Queries

Query data is often passed as part of the URL for a GET request. In order to parse it, the
`util::parse_query()` function can be used. The signature is easy to work with, and flexible.

Queries are automatically split on `&` and `;`.
## Example

```rust
extern crate http_box;

use http_box::util::parse_query;

fn main() {
    parse_query(b"key1;key2=value2",
        |s| {
            if s.has_value() {
                // name with a value
                assert_eq!(b"key2", s.name());
                assert_eq!(b"value2", s.value().unwrap());
            } else {
                // name without a value
                assert_eq!(b"key1", s.name());
            }

            // specifying true indicates that parsing should continue
            true
        }
    );
}
```
