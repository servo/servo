// META: script=/common/utils.js
// META: script=resources/support.sub.js
//
// Spec: https://wicg.github.io/private-network-access/#integration-fetch
//
// These tests mirror fetch.window.js, but use `XmlHttpRequest` instead of
// `fetch()` to perform subresource fetches.
//
// This file covers only those tests that must execute in a non secure context.
// Other tests are defined in: xhr.https.window.js

setup(() => {
  // Making sure we are in a non secure context, as expected.
  assert_false(window.isSecureContext);
});

promise_test(t => xhrTest(t, {
  source: { server: Server.HTTP_LOCAL },
  target: { server: Server.HTTP_LOCAL },
  expected: XhrTestResult.SUCCESS,
}), "local to local: no preflight required.");

promise_test(t => xhrTest(t, {
  source: { server: Server.HTTP_LOCAL },
  target: {
    server: Server.HTTP_PRIVATE,
    behavior: { response: ResponseBehavior.allowCrossOrigin() },
  },
  expected: XhrTestResult.SUCCESS,
}), "local to private: no preflight required.");

promise_test(t => xhrTest(t, {
  source: { server: Server.HTTP_LOCAL },
  target: {
    server: Server.HTTP_PUBLIC,
    behavior: { response: ResponseBehavior.allowCrossOrigin() },
  },
  expected: XhrTestResult.SUCCESS,
}), "local to public: no preflight required.");

promise_test(t => xhrTest(t, {
  source: { server: Server.HTTP_PRIVATE },
  target: {
    server: Server.HTTP_LOCAL,
    behavior: {
      preflight: PreflightBehavior.success(token()),
      response: ResponseBehavior.allowCrossOrigin(),
    },
  },
  expected: XhrTestResult.FAILURE,
}), "private to local: failure.");

promise_test(t => xhrTest(t, {
  source: { server: Server.HTTP_PRIVATE },
  target: { server: Server.HTTP_PRIVATE },
  expected: XhrTestResult.SUCCESS,
}), "private to private: no preflight required.");

promise_test(t => xhrTest(t, {
  source: { server: Server.HTTP_PRIVATE },
  target: {
    server: Server.HTTP_PUBLIC,
    behavior: { response: ResponseBehavior.allowCrossOrigin() },
  },
  expected: XhrTestResult.SUCCESS,
}), "private to public: no preflight required.");

promise_test(t => xhrTest(t, {
  source: { server: Server.HTTP_PUBLIC },
  target: {
    server: Server.HTTP_LOCAL,
    behavior: {
      preflight: PreflightBehavior.success(token()),
      response: ResponseBehavior.allowCrossOrigin(),
    },
  },
  expected: XhrTestResult.FAILURE,
}), "public to local: failure.");

promise_test(t => xhrTest(t, {
  source: { server: Server.HTTP_PUBLIC },
  target: {
    server: Server.HTTP_PRIVATE,
    behavior: {
      preflight: PreflightBehavior.success(token()),
      response: ResponseBehavior.allowCrossOrigin(),
    },
  },
  expected: XhrTestResult.FAILURE,
}), "public to private: failure.");

promise_test(t => xhrTest(t, {
  source: { server: Server.HTTP_PUBLIC },
  target: { server: Server.HTTP_PUBLIC },
  expected: XhrTestResult.SUCCESS,
}), "public to public: no preflight required.");

// These tests verify that documents fetched from the `local` address space yet
// carrying the `treat-as-public-address` CSP directive are treated as if they
// had been fetched from the `public` address space.

promise_test(t => xhrTest(t, {
  source: {
    server: Server.HTTP_LOCAL,
    treatAsPublic: true,
  },
  target: {
    server: Server.HTTP_LOCAL,
    behavior: {
      preflight: PreflightBehavior.success(token()),
      response: ResponseBehavior.allowCrossOrigin(),
    },
  },
  expected: XhrTestResult.FAILURE,
}), "treat-as-public-address to local: failure.");

promise_test(t => xhrTest(t, {
  source: {
    server: Server.HTTP_LOCAL,
    treatAsPublic: true,
  },
  target: {
    server: Server.HTTP_PRIVATE,
    behavior: {
      preflight: PreflightBehavior.success(token()),
      response: ResponseBehavior.allowCrossOrigin(),
    },
  },
  expected: XhrTestResult.FAILURE,
}), "treat-as-public-address to private: failure.");

promise_test(t => xhrTest(t, {
  source: {
    server: Server.HTTP_LOCAL,
    treatAsPublic: true,
  },
  target: {
    server: Server.HTTP_PUBLIC,
    behavior: { response: ResponseBehavior.allowCrossOrigin() },
  },
  expected: XhrTestResult.SUCCESS,
}), "treat-as-public-address to public: no preflight required.");

// These tests verify that HTTPS iframes embedded in an HTTP top-level document
// cannot fetch subresources from less-public address spaces. Indeed, even
// though the iframes have HTTPS origins, they are non-secure contexts because
// their parent is a non-secure context.

promise_test(t => xhrTest(t, {
  source: { server: Server.HTTPS_PRIVATE },
  target: {
    server: Server.HTTPS_LOCAL,
    behavior: {
      preflight: PreflightBehavior.success(token()),
      response: ResponseBehavior.allowCrossOrigin(),
    },
  },
  expected: XhrTestResult.FAILURE,
}), "private https to local: failure.");

promise_test(t => xhrTest(t, {
  source: { server: Server.HTTPS_PUBLIC },
  target: {
    server: Server.HTTPS_LOCAL,
    behavior: {
      preflight: PreflightBehavior.success(token()),
      response: ResponseBehavior.allowCrossOrigin(),
    },
  },
  expected: XhrTestResult.FAILURE,
}), "public https to local: failure.");

promise_test(t => xhrTest(t, {
  source: { server: Server.HTTPS_PUBLIC },
  target: {
    server: Server.HTTPS_PRIVATE,
    behavior: {
      preflight: PreflightBehavior.success(token()),
      response: ResponseBehavior.allowCrossOrigin(),
    },
  },
  expected: XhrTestResult.FAILURE,
}), "public https to private: failure.");
