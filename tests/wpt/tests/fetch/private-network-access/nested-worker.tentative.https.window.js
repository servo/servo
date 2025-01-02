// META: script=/common/utils.js
// META: script=resources/support.sub.js
//
// Spec: https://wicg.github.io/private-network-access/#integration-fetch
//
// These tests check that initial `Worker` script fetches from within worker
// scopes are subject to Private Network Access checks, just like a worker
// script fetches from within document scopes (for non-nested workers). The
// latter are tested in: worker.https.window.js
//
// This file covers only those tests that must execute in a secure context.
// Other tests are defined in: nested-worker.window.js

promise_test(t => nestedWorkerScriptTest(t, {
  source: {
    server: Server.HTTPS_LOCAL,
    treatAsPublic: true,
  },
  target: { server: Server.HTTPS_LOCAL },
  expected: WorkerScriptTestResult.FAILURE,
}), "treat-as-public to local: failure.");

promise_test(t => nestedWorkerScriptTest(t, {
  source: {
    server: Server.HTTPS_PRIVATE,
    treatAsPublic: true,
  },
  target: { server: Server.HTTPS_PRIVATE },
  expected: WorkerScriptTestResult.FAILURE,
}), "treat-as-public to private: failure.");

promise_test(t => nestedWorkerScriptTest(t, {
  source: { server: Server.HTTPS_PUBLIC },
  target: { server: Server.HTTPS_PUBLIC },
  expected: WorkerScriptTestResult.SUCCESS,
}), "public to public: success.");
