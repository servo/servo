// META: global=window,worker
// META: script=/common/get-host-info.sub.js
// META: script=resources/webtransport-test-helpers.sub.js

function set_csp(destination) {
  let meta = document.createElement('meta');
  meta.httpEquiv = 'Content-Security-Policy';
  meta.content = `connect-src ${destination}`;
  return meta;
}

promise_test(async t => {
  let meta = set_csp("'none'");
  document.head.appendChild(meta);

  let wt = new WebTransport(webtransport_url('custom-response.py?:status=200'));

  // Sadly we cannot use promise_rejects_dom as the error constructor is
  // WebTransportError rather than DOMException.
  const e = await wt.ready.catch(e => e);
  await promise_rejects_exactly(t, e, wt.ready, 'ready promise should be rejected');
  await promise_rejects_exactly(t, e, wt.closed, 'closed promise should be rejected');
  assert_equals(e.name, 'WebTransportError', 'WebTransportError');
  assert_true(e instanceof WebTransportError);
  assert_equals(e.source, 'session', 'source');
  assert_equals(e.streamErrorCode, null, 'streamErrorCode');
}, 'WebTransport connection should fail when CSP connect-src is set to none and reject the promises');
