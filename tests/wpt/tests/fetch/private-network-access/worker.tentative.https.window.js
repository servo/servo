// META: script=/common/utils.js
// META: script=resources/support.sub.js
//
// Spec: https://wicg.github.io/private-network-access/#integration-fetch
//
// These tests check that initial `Worker` script fetches in secure contexts are
// exempt from Private Network Access checks because workers can only be fetched
// same-origin and the origin is potentially trustworthy. The only way to test
// this is using the `treat-as-public` CSP directive to artificially place the
// parent document in the `public` IP address space.
//
// This file covers only those tests that must execute in a secure context.
// Other tests are defined in: worker.window.js

promise_test(t => workerScriptTest(t, {
  source: {
    server: Server.HTTPS_LOCAL,
    treatAsPublic: true,
  },
  target: { server: Server.HTTPS_LOCAL },
  expected: WorkerScriptTestResult.SUCCESS,
}), "treat-as-public to local: success.");

promise_test(t => workerScriptTest(t, {
  source: {
    server: Server.HTTPS_PRIVATE,
    treatAsPublic: true,
  },
  target: { server: Server.HTTPS_PRIVATE },
  expected: WorkerScriptTestResult.SUCCESS,
}), "treat-as-public to private: success.");

promise_test(t => workerScriptTest(t, {
  source: { server: Server.HTTPS_PUBLIC },
  target: { server: Server.HTTPS_PUBLIC },
  expected: WorkerScriptTestResult.SUCCESS,
}), "public to public: success.");
