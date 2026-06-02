// META: global=window,worker
// META: script=constants.sub.js

test(() => {
  assert_throws_dom('SecurityError', () => CreateInsecureWebSocket(),
                    'constructor should throw');
}, 'constructing an insecure WebSocket in a secure context should throw');
