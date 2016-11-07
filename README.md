# rust-http-box

![Build: Passing](https://img.shields.io/badge/build-passing-brightgreen.svg)
![dev: 0.1.0](https://img.shields.io/badge/dev-0.1.0-ff69b4.svg)
![license: Apache 2.0](https://img.shields.io/badge/license-Apache%202.0-blue.svg)

## What is http-box?

http-box is a push oriented HTTP parser that processes a single byte at a time. It parses *&[u8]*
slices of data. It's not tied to a networking framework, which makes it suitable for more than
writing a web server or client library.

## Why a separate HTTP parser?

Because HTTP parsing support shouldn't be limited to libraries that are tied to networking layers.

Rustaceans already have multiple HTTP web servers, and multiple HTTP request/response libraries. The
concern with this is that they're all tied to different networking layers, or different parsers,
making it hard to move forward at the most rudimentary level.

Writing an HTTP parser should be a trivial, practically self-documenting task. And it should remain
separate from frameworks that use it.

## Features

- Push oriented and will process a single byte at a time.
- Callback oriented with the ability to halt parsing after each callback.
- Headers are normalized to lower-case.
- Option to parse phases separately from entire requests:
  - Status
  - Headers
  - Multipart / Chunked / URL encoded
- Zero copy philosophy.
- DoS protection is easily supported.
- Fast!

## API Documentation

http://metatomic.io/docs/api/http_box/index.html

## Quick Docs

### Parser

[Parser](http://www.metatomic.io/docs/api/http_box/http1/struct.Parser.html) is the guts of
the library. It provides only necessary components for parsing HTTP data.

It offers 4 parser functions:

**[parse_head()](http://www.metatomic.io/docs/api/http_box/http1/struct.Parser.html#method.parse_head)**

Initiates the parsing of HTTP data. It handles everything prior to the body.

**[parse_chunked()](http://www.metatomic.io/docs/api/http_box/http1/struct.Parser.html#method.parse_chunked)**

Used to parse `chunked` transfer-encoded data. This is for data where the sender doesn't know the
entire length ahead of time.

**[parse_multipart()](http://www.metatomic.io/docs/api/http_box/http1/struct.Parser.html#method.parse_multipart)**

Used to parse multipart data. This is typically non-GET requests that contain files.

**[parse_url_encoded()](http://www.metatomic.io/docs/api/http_box/http1/struct.Parser.html#method.parse_url_encoded)**

Used to parse URL encoded data. This is typically non-GET requests that do not contain files.

### HttpHandler trait

Implementing [HttpHandler](http://www.metatomic.io/docs/api/http_box/http1/trait.HttpHandler.html)
is how you provide a custom callback implementation. It is often necessary to provide multiple
implementations based on which type of data is being parsed: head, chunked transfer-encoded,
multipart, URL encoded, etc. The reason it is necessary to do this is because the chunked
transfer-encoded body type, and the multipart body type both use the header callback functions. Of
course it is possible use [State](http://www.metatomic.io/docs/api/http_box/http1/enum.State.html)
to track state, and perform only the expected actions within the header callback functions. But it
may lead to unwanted clutter.

### Callbacks

In a typical application, callbacks receive arguments that are complete pieces of data. However,
[Parser](http://www.metatomic.io/docs/api/http_box/http1/struct.Parser.html) parses data, and
because of this, it must operate one byte at a type. Moreoever, the data being parsed is often
coming from a network connection, and is received as incomplete pieces of data. To stick to the
zero-copy philosophy, and to avoid buffering, callbacks are executed as frequent as necessary.

### Tracking State

Sometimes multiple states need to work together to produce a single result. A good example of this
is when headers are being parsed. The callback for the header name may be called multiple times in
order to receive the full header name. And the same is true for the header value. It isn't until the
header value is complete, that the header name/value pair can be stored.

This is where the [State](http://www.metatomic.io/docs/api/http_box/http1/enum.State.html) enum
comes into play. You can use this to track the current state when a callback is executed. There is
nothing mysterious about this enum. It's a helper type with the objective of simplifying state
tracking.
