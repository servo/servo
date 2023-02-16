// META: script=/common/subset-tests-by-key.js
// META: script=/common/utils.js
// META: script=resources/support.sub.js
// META: variant=?include=from-local
// META: variant=?include=from-private
// META: variant=?include=from-public
//
// Spec: https://wicg.github.io/private-network-access/#integration-fetch
//
// These tests mirror fetch.https.window.js, but use `XmlHttpRequest` instead of
// `fetch()` to perform subresource fetches. Preflights are tested less
// extensively due to coverage being already provided by `fetch()`.
//
// This file covers only those tests that must execute in a secure context.
// Other tests are defined in: xhr.window.js

setup(() => {
  // Making sure we are in a secure context, as expected.
  assert_true(window.isSecureContext);
});

// Source: secure local context.
//
// All fetches unaffected by Private Network Access.

subsetTestByKey("from-local", promise_test, t => xhrTest(t, {
  source: { server: Server.HTTPS_LOCAL },
  target: { server: Server.HTTPS_LOCAL },
  expected: XhrTestResult.SUCCESS,
}), "local to local: no preflight required.");

subsetTestByKey("from-local", promise_test, t => xhrTest(t, {
  source: { server: Server.HTTPS_LOCAL },
  target: {
    server: Server.HTTPS_PRIVATE,
    behavior: { response: ResponseBehavior.allowCrossOrigin() },
  },
  expected: XhrTestResult.SUCCESS,
}), "local to private: no preflight required.");

subsetTestByKey("from-local", promise_test, t => xhrTest(t, {
  source: { server: Server.HTTPS_LOCAL },
  target: {
    server: Server.HTTPS_PUBLIC,
    behavior: { response: ResponseBehavior.allowCrossOrigin() },
  },
  expected: XhrTestResult.SUCCESS,
}), "local to public: no preflight required.");

// Source: private secure context.
//
// Fetches to the local address space require a successful preflight response
// carrying a PNA-specific header.

subsetTestByKey("from-private", promise_test, t => xhrTest(t, {
  source: { server: Server.HTTPS_PRIVATE },
  target: {
    server: Server.HTTPS_LOCAL,
    behavior: { response: ResponseBehavior.allowCrossOrigin() },
  },
  expected: XhrTestResult.FAILURE,
}), "private to local: failed preflight.");

subsetTestByKey("from-private", promise_test, t => xhrTest(t, {
  source: { server: Server.HTTPS_PRIVATE },
  target: {
    server: Server.HTTPS_LOCAL,
    behavior: {
      preflight: PreflightBehavior.success(token()),
      response: ResponseBehavior.allowCrossOrigin(),
    },
  },
  expected: XhrTestResult.SUCCESS,
}), "private to local: success.");

subsetTestByKey("from-private", promise_test, t => xhrTest(t, {
  source: { server: Server.HTTPS_PRIVATE },
  target: { server: Server.HTTPS_PRIVATE },
  expected: XhrTestResult.SUCCESS,
}), "private to private: no preflight required.");

subsetTestByKey("from-private", promise_test, t => xhrTest(t, {
  source: { server: Server.HTTPS_PRIVATE },
  target: {
    server: Server.HTTPS_PUBLIC,
    behavior: { response: ResponseBehavior.allowCrossOrigin() },
  },
  expected: XhrTestResult.SUCCESS,
}), "private to public: no preflight required.");

// Source: public secure context.
//
// Fetches to the local and private address spaces require a successful
// preflight response carrying a PNA-specific header.

subsetTestByKey("from-public", promise_test, t => xhrTest(t, {
  source: { server: Server.HTTPS_PUBLIC },
  target: {
    server: Server.HTTPS_LOCAL,
    behavior: { response: ResponseBehavior.allowCrossOrigin() },
  },
  expected: XhrTestResult.FAILURE,
}), "public to local: failed preflight.");

subsetTestByKey("from-public", promise_test, t => xhrTest(t, {
  source: { server: Server.HTTPS_PUBLIC },
  target: {
    server: Server.HTTPS_LOCAL,
    behavior: {
      preflight: PreflightBehavior.success(token()),
      response: ResponseBehavior.allowCrossOrigin(),
    },
  },
  expected: XhrTestResult.SUCCESS,
}), "public to local: success.");

subsetTestByKey("from-public", promise_test, t => xhrTest(t, {
  source: { server: Server.HTTPS_PUBLIC },
  target: {
    server: Server.HTTPS_PRIVATE,
    behavior: { response: ResponseBehavior.allowCrossOrigin() },
  },
  expected: XhrTestResult.FAILURE,
}), "public to private: failed preflight.");

subsetTestByKey("from-public", promise_test, t => xhrTest(t, {
  source: { server: Server.HTTPS_PUBLIC },
  target: {
    server: Server.HTTPS_PRIVATE,
    behavior: {
      preflight: PreflightBehavior.success(token()),
      response: ResponseBehavior.allowCrossOrigin(),
    },
  },
  expected: XhrTestResult.SUCCESS,
}), "public to private: success.");

subsetTestByKey("from-public", promise_test, t => xhrTest(t, {
  source: { server: Server.HTTPS_PUBLIC },
  target: { server: Server.HTTPS_PUBLIC },
  expected: XhrTestResult.SUCCESS,
}), "public to public: no preflight required.");
