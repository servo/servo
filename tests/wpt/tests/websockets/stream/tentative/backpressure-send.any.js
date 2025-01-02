// META: script=../../constants.sub.js
// META: script=resources/url-constants.js
// META: global=window,worker
// META: timeout=long
// META: variant=?wss
// META: variant=?wpt_flags=h2

// Allow for this much timer jitter.
const JITTER_ALLOWANCE_MS = 200;

// The amount of buffering a WebSocket connection has is not standardised, but
// it's reasonable to expect that it will not be as large as 8MB.
const MESSAGE_SIZE = 8 * 1024 * 1024;

// In this test, the server WebSocket handler waits 2 seconds, and the browser
// times how long it takes to send the first message.
promise_test(async t => {
  const wss = new WebSocketStream(`${BASEURL}/receive-backpressure`);
  const { writable } = await wss.opened;
  const writer = writable.getWriter();
  const start = performance.now();
  await writer.write(new Uint8Array(MESSAGE_SIZE));
  const elapsed = performance.now() - start;
  assert_greater_than_equal(elapsed, 2000 - JITTER_ALLOWANCE_MS);
}, 'backpressure should be applied to sent messages');
