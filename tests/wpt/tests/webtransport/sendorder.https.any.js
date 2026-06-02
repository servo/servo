// META: global=window,worker
// META: script=/common/get-host-info.sub.js
// META: script=resources/webtransport-test-helpers.sub.js
// META: script=/common/utils.js


promise_test(async t => {
  // Establish a WebTransport session.
  const wt = new WebTransport(webtransport_url('echo.py'));
  await wt.ready;

  // Create a bidirectional stream with sendorder
  const {readable, writable} = await wt.createBidirectionalStream({sendOrder: 3});
  assert_equals(writable.sendOrder, 3);

  // Write a message to the writable end, and close it.
  const writer = writable.getWriter();
  const encoder = new TextEncoder();
  writer.write(encoder.encode('Hello World')).catch(() => {});
  await writer.close();

  // Read the data on the readable end.
  const reply = await read_stream_as_string(readable);

  // Check that the message from the readable end matches the writable end.
  assert_equals(reply, 'Hello World');
}, 'WebTransport client should be able to create and handle a bidirectional stream with sendOrder');

promise_test(async t => {
  // Establish a WebTransport session.
  const wt = new WebTransport(webtransport_url('echo.py'));
  await wt.ready;

  // Create a bidirectional stream with sendorder
  const {readable, writable} = await wt.createBidirectionalStream();
  assert_equals(writable.sendOrder, null);
  // modify it
  writable.sendOrder = 4;
  assert_equals(writable.sendOrder, 4);
}, 'WebTransport client should be able to modify unset sendOrder after stream creation');

promise_test(async t => {
  // Establish a WebTransport session.
  const wt = new WebTransport(webtransport_url('echo.py'));
  await wt.ready;

    // Create a bidirectional stream without sendorder
  const {readable, writable} = await wt.createBidirectionalStream({sendOrder: 3});
  assert_equals(writable.sendOrder, 3);
  // modify it
  writable.sendOrder = 5;
  assert_equals(writable.sendOrder, 5);
  writable.sendOrder = null;
  assert_equals(writable.sendOrder, null);
  // Note: this doesn't verify the underlying stack actually changes priority, just the API
  // for controlling sendOrder
}, 'WebTransport client should be able to modify existing sendOrder after stream creation');
