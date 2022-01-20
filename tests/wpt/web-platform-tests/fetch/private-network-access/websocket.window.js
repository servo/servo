// META: script=resources/support.sub.js
//
// Spec: https://wicg.github.io/private-network-access/#integration-fetch

// These tests verify that websocket connections behave similarly to fetches.
//
// This file covers only those tests that must execute in a non secure context.
// Other tests are defined in: websocket.https.window.js

setup(() => {
  // Making sure we are in a non secure context, as expected.
  assert_false(window.isSecureContext);
});

promise_test(t => websocketTest(t, {
  source: { server: Server.HTTP_LOCAL },
  target: { server: Server.WS_LOCAL },
  expected: WebsocketTestResult.SUCCESS,
}), "local to local: websocket success.");

promise_test(t => websocketTest(t, {
  source: { server: Server.HTTP_PRIVATE },
  target: { server: Server.WS_LOCAL },
  expected: WebsocketTestResult.FAILURE,
}), "private to local: websocket failure.");

promise_test(t => websocketTest(t, {
  source: { server: Server.HTTP_PUBLIC },
  target: { server: Server.WS_LOCAL },
  expected: WebsocketTestResult.FAILURE,
}), "public to local: websocket failure.");

promise_test(t => websocketTest(t, {
  source: {
    server: Server.HTTP_LOCAL,
    treatAsPublic: true,
  },
  target: { server: Server.WS_LOCAL },
  expected: WebsocketTestResult.FAILURE,
}), "treat-as-public to local: websocket failure.");
