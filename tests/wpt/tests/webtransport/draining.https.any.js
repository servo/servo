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
  await wt.draining;

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
