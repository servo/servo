// The file including this must also include /common/get-host-info.sub.js to
// pick up the necessary constants.

const HOST = get_host_info().ORIGINAL_HOST;
const PORT = '{{ports[webtransport-h3][0]}}';
const BASE = `https://${HOST}:${PORT}`;

// Wait for the given number of milliseconds (ms).
function wait(ms) { return new Promise(res => step_timeout(res, ms)); }

// Create URL for WebTransport session.
function webtransport_url(handler) {
  return `${BASE}/webtransport/handlers/${handler}`;
}

// Converts WebTransport stream error code to HTTP/3 error code.
// https://ietf-wg-webtrans.github.io/draft-ietf-webtrans-http3/draft-ietf-webtrans-http3.html#section-4.3
function webtransport_code_to_http_code(n) {
  const first = 0x52e4a40fa8db;
  return first + n + Math.floor(n / 0x1e);
}

// Read all chunks from |readable_stream| and return as an array of arrays
async function read_stream(readable_stream) {
  const reader = readable_stream.getReader();

  let chunks = [];
  while (true) {
    const {value: chunk, done} = await reader.read();
    if (done) {
      break;
    }
    chunks.push(chunk);
  }
  reader.releaseLock();

  return chunks;
}

// Read all chunks from |readable_stream|, decode chunks to a utf-8 string, then
// return the string.
async function read_stream_as_string(readable_stream) {
  const decoder = new TextDecoderStream();
  const decode_stream = readable_stream.pipeThrough(decoder);
  const reader = decode_stream.getReader();

  let chunks = '';
  while (true) {
    const {value: chunk, done} = await reader.read();
    if (done) {
      break;
    }
    chunks += chunk;
  }
  reader.releaseLock();

  return chunks;
}

// Decode all chunks in a given ReadableStream, and parse the data using JSON.
async function read_stream_as_json(readable_stream) {
  const text = await read_stream_as_string(readable_stream);
  return JSON.parse(text);
}

// Check the standard request headers and delete them, leaving any "unique"
// headers to check in the test.
function check_and_remove_standard_headers(headers) {
  assert_equals(headers[':scheme'], 'https');
  delete headers[':scheme'];
  assert_equals(headers[':method'], 'CONNECT');
  delete headers[':method'];
  assert_equals(headers[':authority'], `${HOST}:${PORT}`);
  delete headers[':authority'];
  assert_equals(headers[':path'], '/webtransport/handlers/echo-request-headers.py');
  delete headers[':path'];
  assert_equals(headers[':protocol'], 'webtransport');
  delete headers[':protocol'];
  assert_equals(headers['origin'], `${get_host_info().ORIGIN}`);
  delete headers['origin'];
}

async function query(token) {
  const wt = new WebTransport(webtransport_url(`query.py?token=${token}`));
  try {
    await wt.ready;
    const streams = await wt.incomingUnidirectionalStreams;
    const streams_reader = streams.getReader();
    const { value: readable } = await streams_reader.read();
    streams_reader.releaseLock();

    return await read_stream_as_json(readable);
  } finally {
    wt.close();
  }
}

async function readInto(reader, buffer) {
  let offset = 0;

  while (offset < buffer.byteLength) {
    const {value: view, done} = await reader.read(
        new Uint8Array(buffer, offset, buffer.byteLength - offset));
    buffer = view.buffer;
    if (done) {
      break;
    }
    offset += view.byteLength;
  }

  return buffer;
}

// Opens a new WebTransport connection.
async function openWebTransport(remoteContextHelper) {
  const url = webtransport_url('custom-response.py?:status=200');
  await remoteContextHelper.executeScript((url) => {
    window.testWebTransport = new WebTransport(url);
    return window.testWebTransport.ready;
  }, [url]);
}

// Opens a new WebTransport connection and then close it.
async function openThenCloseWebTransport(remoteContextHelper) {
  const url = webtransport_url('custom-response.py?:status=200');
  await remoteContextHelper.executeScript((url) => {
    window.testWebTransport = new WebTransport(url);
    return window.testWebTransport.ready.then(async () => {
      window.testWebTransport.close();
      await window.testWebTransport.closed;
    });
  }, [url]);
}
