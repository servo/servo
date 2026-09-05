// META: global=window,worker
// META: script=/webtransport/resources/webtransport-test-helpers.sub.js

promise_test(async t => {
  const wt = new WebTransport(webtransport_url('echo-request-headers.py'));
  t.add_cleanup(() => wt.close());
  assert_equals(wt.responseHeaders, null,
                'responseHeaders is null synchronously after construction');
  await wt.ready;
}, 'responseHeaders is null before the connection is established');

promise_test(async t => {
  const wt = new WebTransport(webtransport_url('echo-request-headers.py'));
  t.add_cleanup(() => wt.close());
  await wt.ready;
  assert_true(wt.responseHeaders instanceof Headers,
              'responseHeaders is a Headers object after ready');
}, 'responseHeaders is a Headers object after ready resolves');

promise_test(async t => {
  const wt = new WebTransport(
      webtransport_url('custom-response.py?x-test-header=hello&x-other=world'));
  t.add_cleanup(() => wt.close());
  await wt.ready;
  assert_equals(wt.responseHeaders.get('x-test-header'), 'hello');
  assert_equals(wt.responseHeaders.get('x-other'), 'world');
}, 'responseHeaders exposes server-sent response headers');

promise_test(async t => {
  const wt =
      new WebTransport(webtransport_url('custom-response.py?wt-protocol=foo'));
  t.add_cleanup(() => wt.close());
  await wt.ready;
  assert_false(wt.responseHeaders.has('wt-protocol'),
               'wt-protocol must be stripped from responseHeaders');
}, 'wt-protocol is stripped from responseHeaders');

promise_test(async t => {
  // https://fetch.spec.whatwg.org/#forbidden-response-header-name
  const wt = new WebTransport(
      webtransport_url('custom-response.py?set-cookie=probe%3D1'));
  t.add_cleanup(() => wt.close());
  await wt.ready;
  assert_false(
      wt.responseHeaders.has('set-cookie'),
      'Set-Cookie must not be exposed via WebTransport.responseHeaders');
}, 'Set-Cookie is stripped from responseHeaders');

promise_test(async t => {
  const wt = new WebTransport(webtransport_url('echo-request-headers.py'),
                              {headers: {'x-client-header': 'sent'}});
  t.add_cleanup(() => wt.close());
  await wt.ready;

  const streams = await wt.incomingUnidirectionalStreams;
  const stream_reader = streams.getReader();
  const {value: recv_stream} = await stream_reader.read();
  stream_reader.releaseLock();
  const request_headers = await read_stream_as_json(recv_stream);

  // HTTP/3 requires lowercase field names (RFC 9114 §4.2), so the header
  // should arrive at the server lowercased regardless of the source casing.
  assert_equals(request_headers['x-client-header'], 'sent',
                'custom header forwarded to the server');
  assert_equals(request_headers['X-Client-Header'], undefined,
                'custom header is not forwarded with the original casing');
}, 'headers option forwards a custom header to the server, lowercased');

promise_test(async t => {
  // Cookie is on the Fetch forbidden request-header list; per Fetch spec it
  // is silently dropped rather than throwing, so the connection succeeds but
  // the header must not reach the server.
  const wt = new WebTransport(webtransport_url('echo-request-headers.py'),
                              {headers: {'Cookie': 'probe=forbidden'}});
  t.add_cleanup(() => wt.close());
  await wt.ready;

  const streams = await wt.incomingUnidirectionalStreams;
  const stream_reader = streams.getReader();
  const {value: recv_stream} = await stream_reader.read();
  stream_reader.releaseLock();
  const request_headers = await read_stream_as_json(recv_stream);

  assert_not_equals(
      request_headers['cookie'], 'probe=forbidden',
      'user-supplied Cookie must be silently dropped, not forwarded');
}, 'Forbidden request-header names are silently dropped from the headers option');

test(() => {
  assert_throws_js(TypeError, () => new WebTransport('https://localhost:0/', {
                                headers: {'wt-available-protocols': 'test'}
                              }));
}, 'wt-available-protocols header throws TypeError');

test(() => {
  assert_throws_js(TypeError, () => new WebTransport('https://localhost:0/', {
                                headers: {'WT-Available-Protocols': 'test'}
                              }));
}, 'wt-available-protocols header throws TypeError (upper case)');

test(() => {
  assert_throws_js(TypeError, () => new WebTransport('https://localhost:0/', {
                                headers: {'Wt-AVAILABLE-protocols': 'test'}
                              }));
}, 'wt-available-protocols header throws TypeError (mixed case)');

function constructAndClose(options) {
  const wt = new WebTransport('https://localhost:0/', options);
  // Swallow the unhandled rejections from the unreachable URL.
  wt.ready.catch(() => {});
  wt.closed.catch(() => {});
  wt.close();
}

test(() => {
  constructAndClose({headers: [['x-foo', 'bar'], ['x-baz', 'qux']]});
}, 'HeadersInit sequence-of-sequences form is accepted');

test(() => {
  constructAndClose({headers: [['x-dup', 'one'], ['x-dup', 'two']]});
}, 'Duplicate header names in sequence form are accepted');

test(() => {
  assert_throws_js(TypeError,
                   () => new WebTransport('https://localhost:0/',
                                          {headers: [['only-one-element']]}));
}, 'HeadersInit sequence with wrong inner length throws TypeError');
