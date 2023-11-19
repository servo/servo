// META: script=/common/dispatcher/dispatcher.js
// META: script=/common/utils.js
// META: script=resources/support.sub.js
// META: script=/fenced-frame/resources/utils.js
// META: timeout=long
//
// Spec: https://wicg.github.io/private-network-access/#integration-fetch
//
// These tests verify that contexts can navigate fenced frames to more-public or
// same address spaces without private network access preflight request header.

setup(() => {
  assert_true(window.isSecureContext);
});

// Source: secure local context.
//
// All fetches unaffected by Private Network Access.

promise_test(
    t => fencedFrameTest(t, {
      source: {server: Server.HTTPS_LOCAL},
      target: {server: Server.HTTPS_LOCAL},
      expected: FrameTestResult.SUCCESS,
    }),
    'local to local: no preflight required.');

promise_test(
    t => fencedFrameTest(t, {
      source: {server: Server.HTTPS_LOCAL},
      target: {server: Server.HTTPS_PRIVATE},
      expected: FrameTestResult.SUCCESS,
    }),
    'local to private: no preflight required.');

promise_test(
    t => fencedFrameTest(t, {
      source: {server: Server.HTTPS_LOCAL},
      target: {server: Server.HTTPS_PUBLIC},
      expected: FrameTestResult.SUCCESS,
    }),
    'local to public: no preflight required.');

promise_test(
    t => fencedFrameTest(t, {
      source: {server: Server.HTTPS_PRIVATE},
      target: {server: Server.HTTPS_PRIVATE},
      expected: FrameTestResult.SUCCESS,
    }),
    'private to private: no preflight required.');

promise_test(
    t => fencedFrameTest(t, {
      source: {server: Server.HTTPS_PRIVATE},
      target: {server: Server.HTTPS_PUBLIC},
      expected: FrameTestResult.SUCCESS,
    }),
    'private to public: no preflight required.');

promise_test(
    t => fencedFrameTest(t, {
      source: {server: Server.HTTPS_PUBLIC},
      target: {server: Server.HTTPS_PUBLIC},
      expected: FrameTestResult.SUCCESS,
    }),
    'public to public: no preflight required.');

promise_test(
    t => fencedFrameTest(t, {
      source: {
        server: Server.HTTPS_LOCAL,
        treatAsPublic: true,
      },
      target: {server: Server.HTTPS_PUBLIC},
      expected: FrameTestResult.SUCCESS,
    }),
    'treat-as-public-address to public: no preflight required.');

promise_test(
    t => fencedFrameTest(t, {
      source: {
        server: Server.HTTPS_LOCAL,
        treatAsPublic: true,
      },
      target: {
        server: Server.HTTPS_PUBLIC,
        behavior: {preflight: PreflightBehavior.optionalSuccess(token())}
      },
      expected: FrameTestResult.SUCCESS,
    }),
    'treat-as-public-address to local: optional preflight');
