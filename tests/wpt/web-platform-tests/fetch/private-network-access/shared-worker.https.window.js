// META: script=/common/utils.js
// META: script=resources/support.sub.js
//
// Spec: https://wicg.github.io/private-network-access/#integration-fetch
//
// These tests mirror `Worker` tests, except using `SharedWorker`.
// See also: worker.https.window.js
//
// This file covers only those tests that must execute in a secure context.
// Other tests are defined in: shared-worker.window.js

promise_test(t => sharedWorkerScriptTest(t, {
  source: {
    server: Server.HTTPS_LOCAL,
    treatAsPublic: true,
  },
  target: { server: Server.HTTPS_LOCAL },
  expected: WorkerScriptTestResult.FAILURE,
}), "treat-as-public to local: failed preflight.");

promise_test(t => sharedWorkerScriptTest(t, {
  source: {
    server: Server.HTTPS_LOCAL,
    treatAsPublic: true,
  },
  target: {
    server: Server.HTTPS_LOCAL,
    behavior: { preflight: PreflightBehavior.success(token()) },
  },
  expected: WorkerScriptTestResult.SUCCESS,
}), "treat-as-public to local: success.");

promise_test(t => sharedWorkerScriptTest(t, {
  source: {
    server: Server.HTTPS_PRIVATE,
    treatAsPublic: true,
  },
  target: { server: Server.HTTPS_PRIVATE },
  expected: WorkerScriptTestResult.FAILURE,
}), "treat-as-public to private: failed preflight.");

promise_test(t => sharedWorkerScriptTest(t, {
  source: {
    server: Server.HTTPS_PRIVATE,
    treatAsPublic: true,
  },
  target: {
    server: Server.HTTPS_PRIVATE,
    behavior: { preflight: PreflightBehavior.success(token()) },
  },
  expected: WorkerScriptTestResult.SUCCESS,
}), "treat-as-public to private: success.");

promise_test(t => sharedWorkerScriptTest(t, {
  source: { server: Server.HTTPS_PUBLIC },
  target: { server: Server.HTTPS_PUBLIC },
  expected: WorkerScriptTestResult.SUCCESS,
}), "public to public: success.");
