// META: global=window,worker
// META: script=/common/get-host-info.sub.js
// META: script=resources/webtransport-test-helpers.sub.js
// META: script=/common/utils.js

promise_test(async t => {
  const wt = new WebTransport(webtransport_url('echo.py'));
  await wt.ready;

  t.add_cleanup(() => wt.close());

  assert_true(wt.draining instanceof Promise,
              'draining should be a Promise');

  // The draining promise should not resolve immediately
  let draining_resolved = false;
  wt.draining.then(() => draining_resolved = true);

  // Wait a bit to ensure it doesn't resolve immediately
  await new Promise(resolve => t.step_timeout(resolve, 100));

  assert_false(draining_resolved,
               'draining should not resolve immediately after connection');
}, 'WebTransport.draining should be a Promise and not resolve immediately');


promise_test(async t => {
  const wt = new WebTransport(webtransport_url('echo.py'));
  await wt.ready;

  t.add_cleanup(() => wt.close());

  // Test that draining promise is the same object
  const draining1 = wt.draining;
  const draining2 = wt.draining;

  assert_equals(draining1, draining2,
                'draining should return the same promise object');
}, 'WebTransport.draining should return the same promise');

promise_test(async t => {
  const wt = new WebTransport(webtransport_url('echo.py'));

  // draining should be accessible before ready resolves
  assert_true(wt.draining instanceof Promise,
              'draining should be accessible before ready resolves');

  await wt.ready;
  wt.close();
}, 'WebTransport.draining should be accessible before connection is ready');

promise_test(async t => {
  const wt = new WebTransport(webtransport_url('server-drain.py?drain'));
  await wt.ready;

  t.add_cleanup(() => wt.close());

  // The server initiates draining immediately upon connection
  const result = await wt.draining;
  assert_equals(result, undefined, 'draining resolves with undefined');

  // After draining resolves, we should still be able to open streams
  const stream = await wt.createBidirectionalStream();
 assert_true(stream instanceof WebTransportBidirectionalStream, 'should be able to create stream after draining');

}, 'WebTransport.draining should resolve when server initiates draining');

promise_test(async t => {
  const wt = new WebTransport(webtransport_url('server-drain.py?drain'));
  await wt.ready;

  t.add_cleanup(() => wt.close());

  await wt.draining;

  // Verify we can open multiple streams after draining
  const stream1 = await wt.createBidirectionalStream();
  const stream2 = await wt.createUnidirectionalStream();

  assert_not_equals(stream1, null, 'should be able to create bidirectional stream');
  assert_not_equals(stream2, null, 'should be able to create unidirectional stream');
}, 'WebTransport should allow opening streams after server-initiated draining');

promise_test(async t => {
  const wt = new WebTransport(webtransport_url('server-drain.py?drain'));
  t.add_cleanup(() => wt.close());
  await wt.ready;
  await wt.draining;

  const closed_settled = wt.closed.then(() => 'closed', () => 'closed');
  const winner =
      await Promise.race([closed_settled, wait(100).then(() => 'pending')]);
  assert_equals(winner, 'pending', 'closed must remain pending after draining');
}, 'draining does not settle closed');

promise_test(async t => {
  const wt = new WebTransport(webtransport_url('server-drain.py?drain'));
  t.add_cleanup(() => wt.close());
  await wt.ready;
  await wt.draining;

  const stream = await wt.createUnidirectionalStream();
  const bidi = await wt.createBidirectionalStream();
  const writer = stream.getWriter();
  await writer.write(new Uint8Array([0x41]));
  await writer.close();
  const bidiWriter = bidi.writable.getWriter();
  await bidiWriter.write(new Uint8Array([0x42]));
  await bidiWriter.close();
}, 'streams remain usable after draining');

promise_test(async t => {
  // echo.py never sends a WT_DRAIN_SESSION capsule.
  const wt = new WebTransport(webtransport_url('echo.py'));
  await wt.ready;

  wt.close();
  await wt.closed;

  const draining_settled = wt.draining.then(() => 'settled', () => 'settled');
  const winner =
      await Promise.race([draining_settled, wait(100).then(() => 'pending')]);
  assert_equals(
      winner, 'pending',
      'draining must remain pending when closed without a drain signal');
}, 'draining stays pending when the session closes without draining');
