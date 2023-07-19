// META: global=window,worker
// META: script=constants.sub.js
// META: timeout=long
// META: variant=?default
// META: variant=?wss
// META: variant=?wpt_flags=h2

// This is a repro for Chromium bug https://crbug.com/1286909. It will timeout
// if the bug is present.

// With 0.1 second server-side delay per message, sending 50 messages will take
// around 5 seconds.
const MESSAGES_TO_SEND = 50;

// 65536 is the magic number that triggers the bug, as it precisely fills the
// mojo pipe.
const MESSAGE_SIZE = 65536;

promise_test(async t => {
  const message = new Uint8Array(MESSAGE_SIZE);
  const ws =
        new WebSocket(SCHEME_DOMAIN_PORT + '/receive-many-with-backpressure');
  let opened = false;
  ws.onopen = t.step_func(() => {
    opened = true;
    for (let i = 0; i < MESSAGES_TO_SEND; i++) {
      ws.send(message);
    }
  });
  let responsesReceived = 0;
  ws.onmessage = t.step_func(({data}) => {
    assert_equals(data, String(MESSAGE_SIZE), 'size must match');
    if (++responsesReceived == MESSAGES_TO_SEND) {
      ws.close();
    }
  });
  let resolvePromise;
  const promise = new Promise(resolve => {
    resolvePromise = resolve;
  });
  ws.onclose = t.step_func(({wasClean}) => {
    assert_true(opened, 'connection should have been opened');
    assert_true(wasClean, 'close should be clean');
    resolvePromise();
  });
  return promise;
},
    `sending ${MESSAGES_TO_SEND} messages of size ${MESSAGE_SIZE} with ` +
    'backpressure applied should not hang');
