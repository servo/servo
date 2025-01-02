// META: script=resources/support.sub.js
//
// Spec: https://wicg.github.io/private-network-access/#integration-fetch
//
// These tests verify that websocket connections behave similarly to fetches.
//
// This file covers only those tests that must execute in a secure context.
// Other tests are defined in: websocket.https.window.js

setup(() => {
  // Making sure we are in a secure context, as expected.
  assert_true(window.isSecureContext);
});

promise_test(t => websocketTest(t, {
  source: { server: Server.HTTPS_LOCAL },
  target: { server: Server.WSS_LOCAL },
  expected: WebsocketTestResult.SUCCESS,
}), "local to local: websocket success.");

promise_test(t => websocketTest(t, {
  source: { server: Server.HTTPS_PRIVATE },
  target: { server: Server.WSS_LOCAL },
  expected: WebsocketTestResult.SUCCESS,
}), "private to local: websocket success.");

promise_test(t => websocketTest(t, {
  source: { server: Server.HTTPS_PUBLIC },
  target: { server: Server.WSS_LOCAL },
  expected: WebsocketTestResult.SUCCESS,
}), "public to local: websocket success.");

promise_test(t => websocketTest(t, {
  source: {
    server: Server.HTTPS_LOCAL,
    treatAsPublic: true,
  },
  target: { server: Server.WSS_LOCAL },
  expected: WebsocketTestResult.SUCCESS,
}), "treat-as-public to local: websocket success.");
