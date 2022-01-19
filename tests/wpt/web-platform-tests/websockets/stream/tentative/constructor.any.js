// META: script=../../constants.sub.js
// META: script=resources/url-constants.js
// META: global=window,worker
// META: variant=?wss
// META: variant=?wpt_flags=h2

test(() => {
  assert_throws_js(TypeError, () => new WebSocketStream(),
                'constructor should throw');
}, 'constructing with no URL should throw');

test(() => {
  assert_throws_dom("SyntaxError", () => new WebSocketStream('invalid:'),
                    "constructor should throw");
}, 'constructing with an invalid URL should throw');

test(() => {
  assert_throws_js(TypeError,
                () => new WebSocketStream(`${BASEURL}/`, true),
                "constructor should throw");
}, 'constructing with invalid options should throw');

test(() => {
  assert_throws_js(TypeError,
                () => new WebSocketStream(`${BASEURL}/`, {protocols: 'hi'}),
                "constructor should throw");
}, 'protocols should be required to be a list');

promise_test(async () => {
  const wss = new WebSocketStream(ECHOURL);
  await wss.connection;
  assert_equals(wss.url, ECHOURL, 'url should match');
  wss.close();
}, 'constructing with a valid URL should work');

promise_test(async () => {
  const wss = new WebSocketStream(`${BASEURL}/protocol_array`,
                                  {protocols: ['alpha', 'beta']});
  const { readable, protocol } = await wss.connection;
  assert_equals(protocol, 'alpha', 'protocol should be right');
  const reader = readable.getReader();
  const { value, done } = await reader.read();
  assert_equals(value, 'alpha', 'message contents should match');
  wss.close();
}, 'setting a protocol in the constructor should work');

promise_test(t => {
  const wss = new WebSocketStream(`${BASEURL}/404`);
  return Promise.all([
    promise_rejects_dom(t, 'NetworkError', wss.connection,
                    'connection should reject'),
    promise_rejects_dom(t, 'NetworkError', wss.closed, 'closed should reject')
  ]);
}, 'connection failure should reject the promises');

promise_test(async () => {
  const wss = new WebSocketStream(ECHOURL);
  const { readable, writable, protocol, extensions} = await wss.connection;
  // Verify that |readable| really is a ReadableStream using the getReader()
  // brand check. If it doesn't throw the test passes.
  ReadableStream.prototype.getReader.call(readable);
  // Verify that |writable| really is a WritableStream using the getWriter()
  // brand check. If it doesn't throw the test passes.
  WritableStream.prototype.getWriter.call(writable);
  assert_equals(typeof protocol, 'string', 'protocol should be a string');
  assert_equals(typeof extensions, 'string', 'extensions should be a string');
  wss.close();
}, 'wss.connection should resolve to the right types');
