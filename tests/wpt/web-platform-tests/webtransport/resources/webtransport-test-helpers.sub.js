// The file including this must also include /common/get-host-info.sub.js to
// pick up the necessary constants.

const HOST = get_host_info().ORIGINAL_HOST;
const PORT = '{{ports[webtransport-h3][0]}}';
const BASE = `https://${HOST}:${PORT}`;

// Create URL for WebTransport session.
function webtransport_url(handler) {
  return `${BASE}/webtransport/handlers/${handler}`;
}

// Decode all chunks in a given ReadableStream.
async function read_stream_as_json(stream) {
  const decoder = new TextDecoderStream('utf-8');
  const decode_stream = stream.readable.pipeThrough(decoder);
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

  return JSON.parse(chunks);
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
  assert_equals(headers['datagram-flow-id'], '0');
  delete headers['datagram-flow-id'];
}
