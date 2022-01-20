# Writing H2 Tests

These instructions assume you are already familiar with the testing
infrastructure and know how to write a standard HTTP/1.1 test.

On top of the standard `main` handler that the H1 server offers, the
H2 server also offers support for specific frame handlers in the Python
scripts. Currently there is support for for `handle_headers` and `handle_data`.
Unlike the `main` handler, these  are run whenever the server receives a
HEADERS frame (RequestReceived event) or a DATA frame (DataReceived event).
`main` can still be used, but it will be run after the server has received
the request in its entirety.

Here is what a Python script for a test might look like:
```python
def handle_headers(frame, request, response):
    if request.headers["test"] == "pass":
        response.status = 200
        response.headers.update([('test', 'passed')])
        response.write_status_headers()
    else:
        response.status = 403
        response.headers.update([('test', 'failed')])
        response.write_status_headers()
        response.writer.end_stream()

def handle_data(frame, request, response):
    response.writer.write_data(frame.data[::-1])

def main(request, response):
    response.writer.write_data('\nEnd of File', last=True)
```

The above script is fairly simple:
1. Upon receiving the HEADERS frame, `handle_headers` is run.
    - This checks for a header called 'test' and checks if it is set to 'pass'.
    If true, it will immediately send a response header, otherwise it responds
    with a 403 and ends the stream.
2. Any DATA frames received will then be handled by `handle_data`. This will
simply reverse the data and send it back.
3. Once the request has been fully received, `main` is run which will send
one last DATA frame and signal its the end of the stream.

## Response Writer API ##

The H2Response API is pretty much the same as the H1 variant, the main API
difference lies in the H2ResponseWriter which is accessed through `response.writer`

---

#### `write_headers(self, headers, status_code, status_message=None, stream_id=None, last=False):`
Write a HEADER frame using the H2 Connection object, will only work if the
stream is in a state to send HEADER frames. This will automatically format
the headers so that pseudo headers are at the start of the list and correctly
prefixed with ':'. Since this using the H2 Connection object, it requires that
the stream is in the correct state to be sending this frame.

> <b>Note</b>: Will raise ProtocolErrors if pseudo headers are missing.

- <b>Parameters</b>

    - <b>headers</b>: List of (header, value) tuples
    - <b>status_code</b>: The HTTP status code of the response
    - <b>stream_id</b>: Id of stream to send frame on. Will use the request stream ID if None
    - <b>last</b>: Flag to signal if this is the last frame in stream.

---

#### `write_data(self, item, last=False, stream_id=None):`
Write a DATA frame using the H2 Connection object, will only work if the
stream is in a state to send DATA frames. Uses flow control to split data
into multiple data frames if it exceeds the size that can be in a single frame.
Since this using the H2 Connection object, it requires that the stream is in
the correct state to be sending this frame.

- <b>Parameters</b>

    - <b>item</b>: The content of the DATA frame
    - <b>last</b>: Flag to signal if this is the last frame in stream.
    - <b>stream_id</b>: Id of stream to send frame on. Will use the request stream ID if None

---

#### `write_push(self, promise_headers, push_stream_id=None, status=None, response_headers=None, response_data=None):`
This will write a push promise to the request stream. If you do not provide
headers and data for the response, then no response will be pushed, and you
should send them yourself using the ID returned from this function.

- <b>Parameters</b>
    - <b>promise_headers</b>: A list of header tuples that matches what the client would use to
                        request the pushed response
    - <b>push_stream_id</b>: The ID of the stream the response should be pushed to. If none given, will
                       use the next available id.
    - <b>status</b>: The status code of the response, REQUIRED if response_headers given
    - <b>response_headers</b>: The headers of the response
    - <b>response_data</b>: The response data.

- <b>Returns</b>: The ID of the push stream

---

#### `write_raw_header_frame(self, headers, stream_id=None, end_stream=False, end_headers=False, frame_cls=HeadersFrame):`
Unlike `write_headers`, this does not check to see if a stream is in the
correct state to have HEADER frames sent through to it. It also won't force
the order of the headers or make sure pseudo headers are prefixed with ':'.
It will build a HEADER frame and send it without using the H2 Connection
object other than to HPACK encode the headers.

> <b>Note</b>: The `frame_cls` parameter is so that this class can be reused
by `write_raw_continuation_frame`, as their construction is identical.

- <b>Parameters</b>
    - <b>headers</b>: List of (header, value) tuples
    - <b>stream_id</b>: Id of stream to send frame on. Will use the request stream ID if None
    - <b>end_stream</b>: Set to `True` to add END_STREAM flag to frame
    - <b>end_headers</b>: Set to `True` to add END_HEADERS flag to frame

---

#### `write_raw_data_frame(self, data, stream_id=None, end_stream=False):`
Unlike `write_data`, this does not check to see if a stream is in the correct
state to have DATA frames sent through to it. It will build a DATA frame and
send it without using the H2 Connection object. It will not perform any flow control checks.

- <b>Parameters</b>
    - <b>data</b>: The data to be sent in the frame
    - <b>stream_id</b>: Id of stream to send frame on. Will use the request stream ID if None
    - <b>end_stream</b>: Set to True to add END_STREAM flag to frame

---

#### `write_raw_continuation_frame(self, headers, stream_id=None, end_headers=False):`
This provides the ability to create and write a CONTINUATION frame to the
stream, which is not exposed by `write_headers` as the h2 library handles
the split between HEADER and CONTINUATION internally. Will perform HPACK
encoding on the headers. It also ignores the state of the stream.

This calls `write_raw_data_frame` with `frame_cls=ContinuationFrame` since
the HEADER and CONTINUATION frames are constructed in the same way.

- <b>Parameters</b>:
    - <b>headers</b>: List of (header, value) tuples
    - <b>stream_id</b>: Id of stream to send frame on. Will use the request stream ID if None
    - <b>end_headers</b>: Set to True to add END_HEADERS flag to frame

---

#### `end_stream(self, stream_id=None):`
Ends the stream with the given ID, or the one that request was made on if no ID given.

- <b>Parameters</b>
    - <b>stream_id</b>: Id of stream to send frame on. Will use the request stream ID if None
