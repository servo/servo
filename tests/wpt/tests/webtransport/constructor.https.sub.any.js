// META: global=window,worker
// META: script=resources/webtransport-test-helpers.sub.js
// META: script=/common/utils.js

const BAD_URLS = [
  null,
  '',
  'no-scheme',
  'http://{{domains[nonexistent]}}/' /* scheme is wrong */,
  'quic-transport://{{domains[nonexistent]}}/' /* scheme is wrong */,
  'https:///' /* no host  specified */,
  'https://{{domains[nonexistent]}}/#failing' /* has fragment */,
  'https://{{host}}:999999/' /* invalid port */,
];

for (const url of BAD_URLS) {
  test(() => {
    assert_throws_dom('SyntaxError', () => new WebTransport(url),
                      'constructor should throw');
  }, `WebTransport constructor should reject URL '${url}'`);
}

const BAD_PROTOCOL_LISTS = [
  [''],
  ['\u8AA4'],
  ['test', 'test'],
  ['a'.repeat(513)],
];

for (const protocols of BAD_PROTOCOL_LISTS) {
  test(() => {
    assert_throws_dom('SyntaxError', () => new WebTransport(
                                               webtransport_url(`echo.py`),
                                               { protocols : protocols }),
                      'constructor should throw');
  }, `WebTransport constructor should reject protocol list '${protocols}'`);
}

const OPTIONS = [
  { allowPooling: true },
  { requireUnreliable: true },
  { allowPooling: true, requireUnreliable: true },
  { congestionControl: "default" },
  { congestionControl: "throughput" },
  { congestionControl: "low-latency" },
  { protocols: ["test"] },
  { protocols: ["a", "b", "c"] },
  { allowPooling: true, requireUnreliable: true, congestionControl: "low-latency" },
  // XXX Need to test serverCertificateHashes
];

for (const options of OPTIONS) {
  promise_test(async t => {
    const id = token();
    const wt = new WebTransport(webtransport_url(`client-close.py?token=${id}`), options );
    await wt.ready;
    wt.close();
  }, "WebTransport constructor should allow options " + JSON.stringify(options));
}

promise_test(async t => {
  const wt = new WebTransport('https://{{host}}:0/');

  // Sadly we cannot use promise_rejects_dom as the error constructor is
  // WebTransportError rather than DOMException.
  // We get a possible error, and then make sure wt.ready is rejected with it.
  const e = await wt.ready.catch(e => e);

  await promise_rejects_exactly(t, e, wt.ready, 'ready should be rejected');
  await promise_rejects_exactly(t, e, wt.closed, 'closed should be rejected');
  assert_true(e instanceof WebTransportError);
  assert_equals(e.source, 'session', 'source');
  assert_equals(e.streamErrorCode, null, 'streamErrorCode');
}, 'Connection to port 0 should fail');
