// META: global=window,worker
// META: script=/common/get-host-info.sub.js

const HOST = get_host_info().ORIGINAL_HOST;

const BAD_URLS = [
  null,
  '',
  'no-scheme',
  'https://example.com/' /* scheme is wrong */,
  'quic-transport:///' /* no host  specified */,
  'quic-transport://example.com/#failing' /* has fragment */,
  `quic-transport://${HOST}:999999/` /* invalid port */,
];

for (const url of BAD_URLS) {
  test(() => {
    assert_throws_dom('SyntaxError', () => new QuicTransport(url),
                      'constructor should throw');
  }, `QuicTransport constructor should reject URL '${url}'`);
}

// TODO(ricea): Test CSP.

promise_test(t => {
  const qt = new QuicTransport(`quic-transport://${HOST}:0/`);
  return Promise.all([
    promise_rejects_js(t, TypeError, qt.ready, 'ready promise rejects'),
    promise_rejects_js(t, TypeError, qt.closed, 'closed promise rejects'),
  ]);
}, 'connection to port 0 should fail');
