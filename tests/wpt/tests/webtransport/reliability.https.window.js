// META: script=/webtransport/resources/webtransport-test-helpers.sub.js

promise_test(async t => {
  const wt = new WebTransport(webtransport_url('echo.py'));
  t.add_cleanup(() => wt.close());

  await wt.ready;

  assert_in_array(
      wt.reliability, ['reliable-only', 'supports-unreliable'],
      'reliability reflects the transport mode when ready resolves');
}, 'WebTransport reliability transitions when the connection is ready');
