# rust-http-box

![Build: Passing](https://img.shields.io/badge/build-passing-brightgreen.svg)
![dev: 0.1.0](https://img.shields.io/badge/dev-0.1.0-ff69b4.svg)
![license: Apache 2.0](https://img.shields.io/badge/license-Apache%202.0-blue.svg)

rust-http-box is a fast push/callback oriented HTTP/1.1 parser that can be interrupted during any
callback. This library is not tied to any networking framework.

Errors are handled intelligently letting you know what state the parser was in and which byte
triggered the error when it occurred.

## Progress

Multipart parsing has been wrapped up, but I'm in the process of fleshing out the handler implementation.
If you do not need a handler implementation, and want to work with the raw multipart data, the current state
should suffice.

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
  - Multipart

## Access To:

- [Request](http://www.metatomic.io/docs/api/http_box/handler/struct.HeadersHandler.html#request-example):
  - [Method](http://www.metatomic.io/docs/api/http_box/handler/struct.HeadersHandler.html#method.method)
  - [URL](http://www.metatomic.io/docs/api/http_box/handler/struct.HeadersHandler.html#method.url)
  - [Version](http://www.metatomic.io/docs/api/http_box/handler/struct.HeadersHandler.html#method.version_major)
- [Response](http://www.metatomic.io/docs/api/http_box/handler/struct.HeadersHandler.html#response-example):
  - [Status](http://www.metatomic.io/docs/api/http_box/handler/struct.HeadersHandler.html#method.status)
  - [Status code](http://www.metatomic.io/docs/api/http_box/handler/struct.HeadersHandler.html#method.status_code)
  - [Version](http://www.metatomic.io/docs/api/http_box/handler/struct.HeadersHandler.html#method.version_major)
- [Headers](http://www.metatomic.io/docs/api/http_box/handler/struct.HeadersHandler.html) (quoted and multi-line values are supported):
  - [Cookie name/value](http://www.metatomic.io/docs/api/http_box/handler/struct.HeadersHandler.html#method.cookies)
  - [Header field/value](http://www.metatomic.io/docs/api/http_box/handler/struct.HeadersHandler.html#method.headers)
- [Chunk encoded](http://www.metatomic.io/docs/api/http_box/handler/struct.ChunkedHandler.html):
  - [Length](http://www.metatomic.io/docs/api/http_box/handler/struct.ChunkedHandler.html#method.len)
  - [Extension name/value](http://www.metatomic.io/docs/api/http_box/handler/struct.ChunkedHandler.html#method.extensions)
  - [Trailer field/value](http://www.metatomic.io/docs/api/http_box/handler/struct.ChunkedHandler.html#method.trailers)
  - [Raw data](http://www.metatomic.io/docs/api/http_box/handler/struct.ChunkedHandler.html#example)
- [Multipart](http://www.metatomic.io/docs/api/http_box/handler/struct.MultipartHandler.html):
  - [Field/value](http://www.metatomic.io/docs/api/http_box/handler/struct.MultipartHandler.html#method.fields)
  - [Header field/value](http://www.metatomic.io/docs/api/http_box/handler/struct.MultipartHandler.html#method.headers)
  - File support (working on it)
- [URL encoded](http://www.metatomic.io/docs/api/http_box/handler/struct.UrlEncodedHandler.html):
  - [Field/value](http://www.metatomic.io/docs/api/http_box/handler/struct.UrlEncodedHandler.html#method.fields)

## API Documentation and Examples

http://metatomic.io/docs/api/http_box/index.html
