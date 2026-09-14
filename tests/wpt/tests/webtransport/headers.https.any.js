// META: global=window,worker
// META: script=/webtransport/resources/webtransport-test-helpers.sub.js

async function read_first_incoming_stream_as_json(wt) {
  const streams = await wt.incomingUnidirectionalStreams;
  const stream_reader = streams.getReader();
  const {value: recv_stream} = await stream_reader.read();
  stream_reader.releaseLock();
  return read_stream_as_json(recv_stream);
}

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
  const wt = new WebTransport(
      webtransport_url('custom-response.py?x-test-header=hello'));
  t.add_cleanup(() => wt.close());
  await wt.ready;

  // Verify that the mutating methods actually exist. If they didn't, calling
  // them would throw a TypeError, which would cause the `assert_throws_js`
  // checks below to pass for the wrong reason.
  assert_equals(typeof wt.responseHeaders.delete, 'function');
  assert_equals(typeof wt.responseHeaders.set, 'function');
  assert_equals(typeof wt.responseHeaders.append, 'function');

  assert_throws_js(TypeError, () => wt.responseHeaders.delete('x-test-header'),
                   'delete must throw on immutable responseHeaders');
  assert_throws_js(TypeError,
                   () => wt.responseHeaders.set('x-test-header', 'changed'),
                   'set must throw on immutable responseHeaders');
  assert_throws_js(TypeError,
                   () => wt.responseHeaders.append('x-added', 'nope'),
                   'append must throw on immutable responseHeaders');

  assert_equals(wt.responseHeaders.get('x-test-header'), 'hello',
                'the existing header survives the rejected mutations');
  assert_false(wt.responseHeaders.has('x-added'),
               'no header is added by the rejected append');
}, 'responseHeaders is immutable');

promise_test(async t => {
  const wt =
      new WebTransport(webtransport_url('custom-response.py?wt-protocol=foo'));
  t.add_cleanup(() => wt.close());
  await wt.ready;
  assert_false(wt.responseHeaders.has('wt-protocol'),
               'wt-protocol must be stripped from responseHeaders');
  assert_false([...wt.responseHeaders.keys()].includes('wt-protocol'),
               'wt-protocol must not be exposed via iteration');
}, 'wt-protocol is stripped from responseHeaders');

promise_test(async t => {
  // https://fetch.spec.whatwg.org/#forbidden-response-header-name
  const wt = new WebTransport(webtransport_url(
      'custom-response.py?set-cookie=probe%3D1&x-safe-header=yes'));
  t.add_cleanup(() => wt.close());
  await wt.ready;
  assert_false(
      wt.responseHeaders.has('set-cookie'),
      'Set-Cookie must not be exposed via WebTransport.responseHeaders');
  assert_false([...wt.responseHeaders.keys()].includes('set-cookie'),
               'Set-Cookie must not be exposed via iteration');
  assert_equals(wt.responseHeaders.get('x-safe-header'), 'yes',
                'other response headers are still exposed');
}, 'Set-Cookie is stripped from responseHeaders');

promise_test(async t => {
  const wt = new WebTransport(
      webtransport_url('echo-request-headers.py'),
      {headers: {'X-Client-Header': 'sent', 'x-padded-header': '  padded  '}});
  t.add_cleanup(() => wt.close());
  await wt.ready;

  const request_headers = await read_first_incoming_stream_as_json(wt);

  // HTTP/3 requires lowercase field names (RFC 9114 §4.2), so the header
  // should arrive at the server lowercased regardless of the source casing.
  assert_equals(request_headers['x-client-header'], 'sent',
                'custom header forwarded to the server');
  assert_equals(request_headers['X-Client-Header'], undefined,
                'custom header is not forwarded with the original casing');
  assert_equals(request_headers['x-padded-header'], 'padded',
                'header value is normalized before it is sent');
}, 'headers option forwards a custom header to the server, lowercased');

promise_test(async t => {
  // Cookie, Host and Referer are on the Fetch forbidden request-header list
  // by name; Sec-Fetch-Mode is forbidden by the `Sec-` prefix rule. Forbidden
  // names are silently dropped rather than throwing, so the connection
  // succeeds but none of these headers reach the server.
  const wt = new WebTransport(webtransport_url('echo-request-headers.py'), {
    headers: {
      'Cookie': 'probe=forbidden',
      'Host': 'evil.example.com',
      'Sec-Fetch-Mode': 'navigate',
      'Referer': 'https://evil.example.com/'
    }
  });
  t.add_cleanup(() => wt.close());
  await wt.ready;

  const request_headers = await read_first_incoming_stream_as_json(wt);

  assert_equals(request_headers['cookie'], undefined,
                'user-supplied Cookie must be silently dropped, not forwarded');
  assert_equals(request_headers['host'], undefined,
                'user-supplied Host must be silently dropped, not forwarded');
  assert_equals(
      request_headers['sec-fetch-mode'], undefined,
      'user-supplied Sec-Fetch-Mode must be silently dropped, not forwarded');
  assert_equals(
      request_headers['referer'], undefined,
      'user-supplied Referer must be silently dropped, not forwarded');
}, 'Forbidden request-header names are silently dropped from the headers option');

promise_test(async t => {
  const wt = new WebTransport(webtransport_url('echo-request-headers.py'),
                              {headers: {}});
  t.add_cleanup(() => wt.close());
  await wt.ready;
}, 'An empty headers object is accepted');

test(() => {
  assert_throws_js(TypeError,
                   () => new WebTransport('https://localhost:0/',
                                          {headers: {'bad name': 'value'}}));
}, 'Invalid header name throws TypeError');

test(() => {
  assert_throws_js(TypeError, () => new WebTransport('https://localhost:0/', {
                                headers: {'x-custom': 'bad\r\nvalue'}
                              }));
}, 'Header value containing CR/LF throws TypeError');

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

promise_test(async t => {
  const wt = new WebTransport(webtransport_url('echo-request-headers.py'),
                              {headers: [['x-foo', 'bar'], ['x-baz', 'qux']]});
  t.add_cleanup(() => wt.close());
  await wt.ready;

  const request_headers = await read_first_incoming_stream_as_json(wt);

  assert_equals(request_headers['x-foo'], 'bar');
  assert_equals(request_headers['x-baz'], 'qux');
}, 'HeadersInit sequence-of-sequences form sends every header');

promise_test(async t => {
  const wt =
      new WebTransport(webtransport_url('echo-request-headers.py'),
                       {headers: new Headers({'x-one': '1', 'x-two': '2'})});
  t.add_cleanup(() => wt.close());
  await wt.ready;

  const request_headers = await read_first_incoming_stream_as_json(wt);

  assert_equals(request_headers['x-one'], '1');
  assert_equals(request_headers['x-two'], '2');
}, 'HeadersInit Headers object form sends every header');

promise_test(async t => {
  const wt =
      new WebTransport(webtransport_url('echo-request-headers.py?format=list'),
                       {headers: [['x-dup', 'one'], ['x-dup', 'two']]});
  t.add_cleanup(() => wt.close());
  await wt.ready;

  const request_headers = await read_first_incoming_stream_as_json(wt);

  assert_array_equals(request_headers['x-dup'], ['one', 'two']);
}, 'Duplicate header names in sequence form are all sent, in order');

promise_test(async t => {
  const wt =
      new WebTransport(webtransport_url('echo-request-headers.py?format=list'),
                       {headers: [['foo', 'bar'], ['Foo', 'baz']]});
  t.add_cleanup(() => wt.close());
  await wt.ready;

  const request_headers = await read_first_incoming_stream_as_json(wt);

  assert_array_equals(request_headers['foo'], ['bar', 'baz']);
  assert_equals(request_headers['Foo'], undefined);
}, 'Header names differing only in case are lowercased and both sent');

test(() => {
  assert_throws_js(
      TypeError,
      () => new WebTransport('https://localhost:0/', {headers: [[]]}));
}, 'HeadersInit sequence with an empty inner sequence throws TypeError');

test(() => {
  assert_throws_js(TypeError,
                   () => new WebTransport('https://localhost:0/',
                                          {headers: [['only-one-element']]}));
}, 'HeadersInit sequence with one inner element throws TypeError');

test(() => {
  assert_throws_js(TypeError, () => new WebTransport('https://localhost:0/', {
                                headers: [['one', 'two', 'three']]
                              }));
}, 'HeadersInit sequence with three inner elements throws TypeError');
