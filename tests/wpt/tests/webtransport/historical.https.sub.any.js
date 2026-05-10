// META: global=window,worker
// META: script=/common/get-host-info.sub.js
// META: script=resources/webtransport-test-helpers.sub.js

async function get_wt() {
  const wt = new WebTransport(webtransport_url('{{domains[nonexistent]}}'));
  // `ready` and `closed` promises will be rejected due to connection error.
  // Catches them to avoid unhandled rejections.
  wt.ready.catch(() => {});
  wt.closed.catch(() => {});
  return wt;
}

promise_test(async t => {
  // https://github.com/w3c/webtransport/commit/3e37d39bb4399935f8c88018fe3008698cad7862
  const wt = await get_wt();
  assert_false('writable' in wt.datagrams);
}, 'WebTransportDatagramDuplexStream#writable is removed');

promise_test(async t => {
  // https://github.com/w3c/webtransport/commit/56eb7e184c1c91fda932557f3ccfc44fc2187503
  const wt = await get_wt();
  assert_false('incomingHighWaterMark' in wt.datagrams);
}, 'WebTransportDatagramDuplexStream#incomingHighWaterMark is removed');

promise_test(async t => {
  // https://github.com/w3c/webtransport/commit/5e10b91e73bd409a8577a4d2264c53b8dcfdb353
  const wt = await get_wt();
  assert_false('outgoingHighWaterMark' in wt.datagrams);
}, 'WebTransportDatagramDuplexStream#outgoingHighWaterMark is removed');
