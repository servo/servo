# WebTransport in web-platform-tests

This document describes [WebTransport](https://datatracker.ietf.org/wg/webtrans/documents/) support in web-platform-tests.

## WebTransport over HTTP/3
`tools/webtransport` provides a simple
[WebTransport over HTTP/3](https://datatracker.ietf.org/doc/draft-ietf-webtrans-http3/) server for testing. The server interprets the underlying protocols (WebTransport, HTTP/3 and QUIC) and manages webtransport sessions. When the server receives a request (extended CONNECT method) from a client the server looks up a corresponding webtransport handler based on the `:path` header value, then delegates actual tasks to the handler. Handlers are typically located under `webtransport/handlers`.

### Handlers

A WebTransport handler is a python script which contains callback functions. Callback functions are called every time a WebTransport event happens. Definitions of all callback can be found the [APIs section](#APIs).

The following is an example handler which echos back received data.

```python
def stream_data_received(session, stream_id: int, data: bytes, stream_ended: bool):
    if session.stream_is_unidirectional(stream_id):
        return
    session.send_stream_data(stream_id, data)


def datagram_received(session, data: bytes):
    session.send_datagram(data)
```

`session` is a `WebTransportSession` object that represents a WebTransport over HTTP/3 session. It provides APIs to handle the session.

### Handler APIs

#### `connection_received(request_headers, response_headers)`
Called whenever an extended CONNECT method is received.

- <b>Parameters</b>

  - <b>request_headers</b>: The request headers received from the peer.
  - <b>response_headers</b>: The response headers which will be sent to the peer `:status` is set to 200 when it isn't specified.

---

#### `session_established(session)`
Called whenever a WebTransport session is established.

- <b>Parameters</b>

  - <b>session</b>: A WebTransport session object.

---

#### `stream_data_received(session, stream_id, data, stream_ended)`
Called whenever data is received on a WebTransport stream.

- <b>Parameters</b>

  - <b>session</b>: A WebTransport session object.
  - <b>stream_id</b>: The ID of the stream.
  - <b>data</b>: The received data.
  - <b>stream_ended</b>: Whether the stream is ended.

---

#### `datagram_received(session, data)`
Called whenever a datagram is received on a WebTransport session.

- <b>Parameters</b>

  - <b>session</b>: A WebTransport session object.
  - <b>data</b>: The received data.

---

#### `stream_reset(session, stream_id, error_code)`
Called whenever a datagram is reset with RESET_STREAM.

- <b>Parameters</b>

  - <b>session</b>: A WebTransport session object.
  - <b>stream_id</b>: The ID of the stream.
  - <b>error_code</b>: The reason of the reset.
