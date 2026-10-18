// META: global=window,worker
// META: script=resources/webtransport-test-helpers.sub.js

promise_test(async t => {
  // https://w3c.github.io/webtransport/#process-a-webtransport-fetch-response
  // Step 6: "wt-protocol" is a negotiation-only header and must be deleted
  // from responseHeaders before it is exposed to script (the negotiated
  // protocol itself is exposed via WebTransport.protocol instead).
  const options = { protocols: ['a', 'b', 'c'] };
  const wt = new WebTransport(
      webtransport_url('custom-response.py?wt-protocol="b"'), options);
  t.add_cleanup(() => wt.close());
  await wt.ready;

  assert_equals(wt.protocol, 'b');
  assert_not_equals(wt.responseHeaders, null);
  assert_equals(wt.responseHeaders.get('wt-protocol'), null,
                'wt-protocol must not be exposed via responseHeaders');
}, 'wt-protocol is stripped from responseHeaders');

promise_test(async t => {
  // https://w3c.github.io/webtransport/#process-a-webtransport-fetch-response
  // Step 7: transport.[[ResponseHeaders]] is an immutable Headers object.
  const wt = new WebTransport(webtransport_url('echo.py'));
  t.add_cleanup(() => wt.close());
  await wt.ready;

  assert_not_equals(wt.responseHeaders, null);
  assert_throws_js(TypeError, () => wt.responseHeaders.set('x-custom', 'evil'));
  assert_throws_js(TypeError, () => wt.responseHeaders.append('x-custom', 'evil'));
  assert_throws_js(TypeError, () => wt.responseHeaders.delete('x-custom'));
}, 'responseHeaders is immutable');
