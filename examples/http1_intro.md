# Parser

[http_box::http1::Parser](https://docs.rs/http-box/0.1.3/http_box/http1/struct.Parser.html)
provides the guts of the HTTP/1.x parsing capabilities.

# HttpHandler

Implementing
[http_box::http1::HttpHandler](https://docs.rs/http-box/0.1.3/http_box/http1/trait.HttpHandler.html)
is how you provide a custom callback implementation. You can implement none, or
as many functions as you want.

Each `Parser` instance can be tied to only one instance of `HttpHandler`. Often
times, it's easiest to provide an `HttpHandler` implementation for each type of
data being processed: head, chunked transfer-encoded, multipart, URL encoded. In
doing so, you will also have a `Parser` instance for each of your `HttpHandler`
implementations. This helps keep code clean at the cost of a bit more memory
being used by multiple `Parser` instances. This is not to say that you are
unable to write a single `HttpHandler` implementation that handles all methods
of parsing.

# Callbacks

In a typical application, callbacks receive arguments that are complete pieces
of data. However, `Parser` parses data byte-by-byte, and because of this, it can
operate one byte at a time. Moreover, the data being parsed is often coming from
a network connection, and is received as incomplete segments of data. To stick
to the zero-copy philosophy, and to avoid buffering, callbacks are executed as
frequent as necessary.

# Tracking State

Sometimes multiple states need to work together to produce a single result. A
good example of this is when headers are being parsed. The callback for the
header name may be called multiple times in order to receive the full header
name. And the same is true for the header value. It isn't until the header value
is complete, that the header name/value pair can be collectively handled.

This is where the
[http_box::http1::State](https://docs.rs/http-box/0.1.3/http_box/http1/enum.State.html)
enum comes into play. You can use this to track the current state when a
callback is executed. There is nothing mysterious about this enum. It's just a
helper type with the objective of simplifying state tracking.
