// META: global=window,worker
// META: script=/common/get-host-info.sub.js
// META: script=resources/webtransport-test-helpers.sub.js

function validate_rtt_stats(stats) {
  // The assumption below is that the RTT to localhost is under 5 seconds,
  // which is fairly generous.
  assert_greater_than(stats.minRtt, 0, "minRtt");
  assert_less_than(stats.minRtt, 5 * 1000, "minRtt");
  assert_greater_than(stats.smoothedRtt, 0, "smoothedRtt");
  assert_less_than(stats.smoothedRtt, 5 * 1000, "smoothedRtt");
}

promise_test(async t => {
  const wt = new WebTransport(webtransport_url('echo.py'));
  await wt.ready;
  const stats = await wt.getStats();
  validate_rtt_stats(stats);
  assert_equals(stats.datagrams.expiredOutgoing, 0);
  assert_equals(stats.datagrams.droppedIncoming, 0);
  assert_equals(stats.datagrams.lostOutgoing, 0);
}, "WebTransport client should be able to provide stats after connection has been established");

promise_test(async t => {
  const wt = new WebTransport(webtransport_url('echo.py'));
  await wt.ready;
  wt.close();

  const stats = await wt.getStats();
  validate_rtt_stats(stats);
}, "WebTransport client should be able to provide stats after connection has been closed");

promise_test(async t => {
  const wt = new WebTransport(webtransport_url('echo.py'));
  await wt.ready;
  const statsPromise = wt.getStats();
  wt.close();

  const stats = await statsPromise;
  validate_rtt_stats(stats);
}, "WebTransport client should be able to provide stats requested right before connection has been closed");

promise_test(async t => {
  const wt = new WebTransport(webtransport_url('echo.py'));
  const stats = await wt.getStats();
  validate_rtt_stats(stats);
}, "WebTransport client should be able to provide valid stats when requested before connection established");

promise_test(async t => {
  const wt = new WebTransport("https://webtransport.invalid/");
  wt.ready.catch(e => {});
  wt.closed.catch(e => {});
  const error = await wt.getStats().catch(e => e);
  assert_equals(error.code, DOMException.INVALID_STATE_ERR);
  const error2 = await wt.getStats().catch(e => e);
  assert_equals(error2.code, DOMException.INVALID_STATE_ERR);
}, "WebTransport client should throw an error when stats are requested for a failed connection");

promise_test(async t => {
  const wt = new WebTransport(webtransport_url('echo.py'));
  await wt.ready;
  const stats1 = wt.getStats();
  const stats2 = wt.getStats();
  assert_true(stats1 != stats2, "different promise returned for different getStats() calls");
  validate_rtt_stats(await stats1);
  validate_rtt_stats(await stats2);
}, "WebTransport client should be able to handle multiple concurrent stats requests");

promise_test(async t => {
  const wt = new WebTransport(webtransport_url('echo.py'));
  await wt.ready;
  const stats1 = await wt.getStats();
  validate_rtt_stats(stats1);
  const stats2 = await wt.getStats();
  validate_rtt_stats(stats2);
}, "WebTransport client should be able to handle multiple sequential stats requests");

promise_test(async t => {
  const wt = new WebTransport(webtransport_url('echo.py'));
  await wt.ready;

  const numDatagrams = 64;
  wt.datagrams.incomingHighWaterMark = 4;

  const writer = wt.datagrams.writable.getWriter();
  const encoder = new TextEncoder();
  const promises = [];
  while (promises.length < numDatagrams) {
    const token = promises.length.toString();
    promises.push(writer.write(encoder.encode(token)));
  }
  await Promise.all(promises);

  const maxAttempts = 40;
  let stats;
  for (let i = 0; i < maxAttempts; i++) {
    wait(50);
    stats = await wt.getStats();
    if (stats.datagrams.droppedIncoming > 0) {
      break;
    }
  }
  assert_greater_than(stats.datagrams.droppedIncoming, 0);
  assert_less_than_equal(stats.datagrams.droppedIncoming,
                         numDatagrams - wt.datagrams.incomingHighWaterMark);
}, "WebTransport client should be able to provide droppedIncoming values for datagrams");
