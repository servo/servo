// META: script=resources/support.js
// META: script=resources/ports.sub.js
//
// Spec: https://wicg.github.io/private-network-access/#integration-fetch
//
// This file covers only those tests that must execute in a non secure context.
// Other tests are defined in: secure-context.window.js

setup(() => {
  // Making sure we are in a non secure context, as expected.
  assert_false(window.isSecureContext);
});

// These tests verify that non-secure contexts cannot fetch subresources from
// less-public address spaces, and can fetch them otherwise.

promise_test(t => fetchTest(t, {
  source: { port: kPorts.httpLocal },
  target: { port: kPorts.httpLocal },
  expected: kFetchTestResult.success,
}), "Local non-secure context can fetch local subresource.");

promise_test(t => fetchTest(t, {
  source: { port: kPorts.httpLocal },
  target: { port: kPorts.httpPrivate },
  expected: kFetchTestResult.success,
}), "Local non-secure context can fetch private subresource.");

promise_test(t => fetchTest(t, {
  source: { port: kPorts.httpLocal },
  target: { port: kPorts.httpPublic },
  expected: kFetchTestResult.success,
}), "Local non-secure context can fetch public subresource.");

promise_test(t => fetchTest(t, {
  source: { port: kPorts.httpPrivate },
  target: { port: kPorts.httpLocal },
  expected: kFetchTestResult.failure,
}), "Private non-secure context cannot fetch local subresource.");

promise_test(t => fetchTest(t, {
  source: { port: kPorts.httpPrivate },
  target: { port: kPorts.httpPrivate },
  expected: kFetchTestResult.success,
}), "Private non-secure context can fetch private subresource.");

promise_test(t => fetchTest(t, {
  source: { port: kPorts.httpPrivate },
  target: { port: kPorts.httpPublic },
  expected: kFetchTestResult.success,
}), "Private non-secure context can fetch public subresource.");

promise_test(t => fetchTest(t, {
  source: { port: kPorts.httpPublic },
  target: { port: kPorts.httpLocal },
  expected: kFetchTestResult.failure,
}), "Public non-secure context cannot fetch local subresource.");

promise_test(t => fetchTest(t, {
  source: { port: kPorts.httpPublic },
  target: { port: kPorts.httpPrivate },
  expected: kFetchTestResult.failure,
}), "Public non-secure context cannot fetch private subresource.");

promise_test(t => fetchTest(t, {
  source: { port: kPorts.httpPublic },
  target: { port: kPorts.httpPublic },
  expected: kFetchTestResult.success,
}), "Public non-secure context can fetch public subresource.");

// These tests verify that documents fetched from the `local` address space yet
// carrying the `treat-as-public-address` CSP directive are treated as if they
// had been fetched from the `public` address space.

promise_test(t => fetchTest(t, {
  source: {
    port: kPorts.httpLocal,
    treatAsPublicAddress: true,
  },
  target: { port: kPorts.httpLocal },
  expected: kFetchTestResult.failure,
}), "Treat-as-public-address non-secure context cannot fetch local subresource.");

promise_test(t => fetchTest(t, {
  source: {
    port: kPorts.httpLocal,
    treatAsPublicAddress: true,
  },
  target: { port: kPorts.httpPrivate },
  expected: kFetchTestResult.failure,
}), "Treat-as-public-address non-secure context cannot fetch private subresource.");

promise_test(t => fetchTest(t, {
  source: {
    port: kPorts.httpLocal,
    treatAsPublicAddress: true,
  },
  target: { port: kPorts.httpPublic },
  expected: kFetchTestResult.success,
}), "Treat-as-public-address non-secure context can fetch public subresource.");

// These tests verify that HTTPS iframes embedded in an HTTP top-level document
// cannot fetch subresources from less-public address spaces. Indeed, even
// though the iframes have HTTPS origins, they are non-secure contexts because
// their parent is a non-secure context.

promise_test(t => fetchTest(t, {
  source: {
    protocol: "https:",
    port: kPorts.httpsPrivate,
  },
  target: {
    protocol: "https:",
    port: kPorts.httpsLocal,
  },
  expected: kFetchTestResult.failure,
}), "Private HTTPS non-secure context cannot fetch local subresource.");

promise_test(t => fetchTest(t, {
  source: {
    protocol: "https:",
    port: kPorts.httpsPublic,
  },
  target: {
    protocol: "https:",
    port: kPorts.httpsLocal,
  },
  expected: kFetchTestResult.failure,
}), "Public HTTPS non-secure context cannot fetch local subresource.");

promise_test(t => fetchTest(t, {
  source: {
    protocol: "https:",
    port: kPorts.httpsPublic,
  },
  target: {
    protocol: "https:",
    port: kPorts.httpsPrivate,
  },
  expected: kFetchTestResult.failure,
}), "Public HTTPS non-secure context cannot fetch private subresource.");

// These tests verify that websocket connections behave similarly to fetches.

promise_test(t => websocketTest(t, {
  source: {
    port: kPorts.httpLocal,
  },
  target: {
    protocol: "ws:",
    port: kPorts.wsLocal,
  },
  expected: kWebsocketTestResult.success,
}), "Local non-secure context can open connection to ws://localhost.");

promise_test(t => websocketTest(t, {
  source: {
    port: kPorts.httpPrivate,
  },
  target: {
    protocol: "ws:",
    port: kPorts.wsLocal,
  },
  expected: kWebsocketTestResult.failure,
}), "Private non-secure context cannot open connection to ws://localhost.");

promise_test(t => websocketTest(t, {
  source: {
    port: kPorts.httpPublic,
  },
  target: {
    protocol: "ws:",
    port: kPorts.wsLocal,
  },
  expected: kWebsocketTestResult.failure,
}), "Public non-secure context cannot open connection to ws://localhost.");

promise_test(t => websocketTest(t, {
  source: {
    port: kPorts.httpLocal,
    treatAsPublicAddress: true,
  },
  target: {
    protocol: "ws:",
    port: kPorts.wsLocal,
  },
  expected: kWebsocketTestResult.failure,
}), "Treat-as-public non-secure context cannot open connection to ws://localhost.");
