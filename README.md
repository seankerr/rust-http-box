# rust-http-box

![Build: Passing](https://img.shields.io/badge/build-passing-brightgreen.svg)
![dev: 0.1.0](https://img.shields.io/badge/dev-0.1.0-ff69b4.svg)
![license: Apache 2.0](https://img.shields.io/badge/license-Apache%202.0-blue.svg)

rust-http-box is a fast push/callback oriented HTTP/1.1 (HTTP/2.0 coming soon) parser that works
only with slices of data, and never copies parsed data. Because of this, it is
possible to parse HTTP data one byte at a time. Parsing can be interrupted during any callback,
and at the end of each parsed slice.

This is purely an HTTP parsing library and is not tied to any networking framework. Use it to parse
stored HTTP request logs, test data, or to write a server and/or client.

Errors are handled intelligently letting you know what state the parser was in and which byte
triggered the error when it occurred.

## Progress

I'm in the stage of refactoring filenames, types, and overall locations of everything in the API.
Aside from this, the current changes are also in progress:

- MultipartHandler (the entire thing)

## Features

- Understands persistent requests
- Easily upgradable from HTTP/1.1 parsing to HTTP/2.0 in the same stream
- Header field, cookie name, chunk extension name normalization to lower-case
- Parses:
  - Requests
  - Responses
  - Headers
  - Cookies
  - Field values; e.g. 'multipart/form-data; boundary="--ABCDEFG"'
  - Chunk encoded data
  - URL encoded data
  - Query strings
  - Multipart (in the works)

## Access To:

- [Request](http://www.metatomic.org/docs/api/http_box/handler/struct.HeadersHandler.html#request-example):
  - [Method](http://www.metatomic.org/docs/api/http_box/handler/struct.HeadersHandler.html#method.get_method)
  - [URL](http://www.metatomic.org/docs/api/http_box/handler/struct.HeadersHandler.html#method.get_url)
  - [Version](http://www.metatomic.org/docs/api/http_box/handler/struct.HeadersHandler.html#method.get_version_major)
- [Response](http://www.metatomic.org/docs/api/http_box/handler/struct.HeadersHandler.html#response-example):
  - [Status](http://www.metatomic.org/docs/api/http_box/handler/struct.HeadersHandler.html#method.get_status)
  - [Status code](http://www.metatomic.org/docs/api/http_box/handler/struct.HeadersHandler.html#method.get_status_code)
  - [Version](http://www.metatomic.org/docs/api/http_box/handler/struct.HeadersHandler.html#method.get_version_major)
- [Headers](http://www.metatomic.org/docs/api/http_box/handler/struct.HeadersHandler.html) (quoted and multi-line values are supported):
  - [Cookie name/value](http://www.metatomic.org/docs/api/http_box/handler/struct.HeadersHandler.html#method.get_cookies)
  - [Header field/value](http://www.metatomic.org/docs/api/http_box/handler/struct.HeadersHandler.html#method.get_headers)
- [Chunk encoded](http://www.metatomic.org/docs/api/http_box/handler/struct.ChunkedHandler.html):
  - [Length](http://www.metatomic.org/docs/api/http_box/handler/struct.ChunkedHandler.html#method.get_length)
  - [Extension name/value](http://www.metatomic.org/docs/api/http_box/handler/struct.ChunkedHandler.html#method.get_extensions)
  - [Trailer field/value](http://www.metatomic.org/docs/api/http_box/handler/struct.ChunkedHandler.html#method.get_trailers)
  - [Raw data](http://www.metatomic.org/docs/api/http_box/handler/struct.ChunkedHandler.html#example)
- Multipart (in the works)
  - Header field/value
  - File support
- [URL encoded](http://www.metatomic.org/docs/api/http_box/handler/struct.UrlEncodedHandler.html):
  - [Field/value](http://www.metatomic.org/docs/api/http_box/handler/struct.UrlEncodedHandler.html#method.get_fields)

## Performance

Currently rust-http-box is on par with the speeds seen from [fast-http](https://github.com/fukamachi/fast-http),
a Common Lisp HTTP parser, and significantly faster than the Joyant/NodeJS HTTP parser.

## API Documentation and Examples

http://metatomic.org/docs/api/http_box/index.html
