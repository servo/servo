// META: global=window,worker
// META: script=resources/webtransport-test-helpers.sub.js

function set_csp(destination) {
  let meta = document.createElement('meta');
  meta.httpEquiv = 'Content-Security-Policy';
  meta.content = `connect-src ${destination}`;
  return meta;
}

promise_test(async t => {
 const handler_url = webtransport_url('custom-response.py?:status=200');
 let meta = set_csp(new URL(handler_url).origin);
 document.head.appendChild(meta);

  let wt = new WebTransport(handler_url);
  await wt.ready;
}, 'WebTransport connection should succeed when CSP connect-src destination is set to the page');
