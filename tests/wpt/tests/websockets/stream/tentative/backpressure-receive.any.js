// META: script=../../constants.sub.js
// META: script=resources/url-constants.js
// META: global=window,worker
// META: timeout=long
// META: variant=?wss
// META: variant=?wpt_flags=h2

// Allow for this much timer jitter.
const JITTER_ALLOWANCE_MS = 200;
const LARGE_MESSAGE_COUNT = 16;

// This test works by using a server WebSocket handler which sends a large
// message, and then sends a second message with the time it measured the first
// message taking. On the browser side, we wait 2 seconds before reading from
// the socket. This should ensure it takes at least 2 seconds to finish sending
// the large message.
promise_test(async t => {
  const wss = new WebSocketStream(`${BASEURL}/send-backpressure`);
  const { readable } = await wss.opened;
  const reader = readable.getReader();

  // Create backpressure for 2 seconds.
  await new Promise(resolve => t.step_timeout(resolve, 2000));

  // Skip the empty message used to fill the readable queue.
  await reader.read();

  // Skip the large messages.
  for (let i = 0; i < LARGE_MESSAGE_COUNT; ++i) {
    await reader.read();
  }

  // Read the time it took.
  const { value, done } = await reader.read();

  // A browser can pass this test simply by being slow. This may be a source of
  // flakiness for browsers that do not implement backpressure properly.
  assert_greater_than_equal(Number(value), 2 - JITTER_ALLOWANCE_MS / 1000,
                            'data send should have taken at least 2 seconds');
}, 'backpressure should be applied to received messages');
