// META: script=/common/utils.js
// META: script=resources/support.sub.js
//
// Spec: https://wicg.github.io/private-network-access/#integration-fetch
//
// These tests check that fetches from within `Worker` scripts are subject to
// Private Network Access checks, just like fetches from within documents.
//
// This file covers only those tests that must execute in a secure context.
// Other tests are defined in: worker-fetch.window.js

promise_test(t => workerFetchTest(t, {
  source: { server: Server.HTTPS_LOCAL },
  target: { server: Server.HTTPS_LOCAL },
  expected: WorkerFetchTestResult.SUCCESS,
}), "local to local: success.");

promise_test(t => workerFetchTest(t, {
  source: { server: Server.HTTPS_PRIVATE },
  target: {
    server: Server.HTTPS_LOCAL,
    behavior: { response: ResponseBehavior.allowCrossOrigin() },
  },
  expected: WorkerFetchTestResult.FAILURE,
}), "private to local: failed preflight.");

promise_test(t => workerFetchTest(t, {
  source: { server: Server.HTTPS_PRIVATE },
  target: {
    server: Server.HTTPS_LOCAL,
    behavior: {
      preflight: PreflightBehavior.success(token()),
      response: ResponseBehavior.allowCrossOrigin(),
    },
  },
  expected: WorkerFetchTestResult.SUCCESS,
}), "private to local: success.");

promise_test(t => workerFetchTest(t, {
  source: { server: Server.HTTPS_PRIVATE },
  target: { server: Server.HTTPS_PRIVATE },
  expected: WorkerFetchTestResult.SUCCESS,
}), "private to private: success.");

promise_test(t => workerFetchTest(t, {
  source: { server: Server.HTTPS_PUBLIC },
  target: {
    server: Server.HTTPS_LOCAL,
    behavior: { response: ResponseBehavior.allowCrossOrigin() },
  },
  expected: WorkerFetchTestResult.FAILURE,
}), "public to local: failed preflight.");

promise_test(t => workerFetchTest(t, {
  source: { server: Server.HTTPS_PUBLIC },
  target: {
    server: Server.HTTPS_LOCAL,
    behavior: {
      preflight: PreflightBehavior.success(token()),
      response: ResponseBehavior.allowCrossOrigin(),
    },
  },
  expected: WorkerFetchTestResult.SUCCESS,
}), "public to local: success.");

promise_test(t => workerFetchTest(t, {
  source: { server: Server.HTTPS_PUBLIC },
  target: {
    server: Server.HTTPS_PRIVATE,
    behavior: { response: ResponseBehavior.allowCrossOrigin() },
  },
  expected: WorkerFetchTestResult.FAILURE,
}), "public to private: failed preflight.");

promise_test(t => workerFetchTest(t, {
  source: { server: Server.HTTPS_PUBLIC },
  target: {
    server: Server.HTTPS_PRIVATE,
    behavior: {
      preflight: PreflightBehavior.success(token()),
      response: ResponseBehavior.allowCrossOrigin(),
    },
  },
  expected: WorkerFetchTestResult.SUCCESS,
}), "public to private: success.");

promise_test(t => workerFetchTest(t, {
  source: { server: Server.HTTPS_PUBLIC },
  target: { server: Server.HTTPS_PUBLIC },
  expected: WorkerFetchTestResult.SUCCESS,
}), "public to public: success.");

promise_test(t => workerFetchTest(t, {
  source: {
    server: Server.HTTPS_LOCAL,
    treatAsPublic: true,
  },
  target: { server: Server.HTTPS_LOCAL },
  expected: WorkerFetchTestResult.FAILURE,
}), "treat-as-public to local: failed preflight.");

promise_test(t => workerFetchTest(t, {
  source: {
    server: Server.HTTPS_LOCAL,
    treatAsPublic: true,
  },
  target: {
    server: Server.HTTPS_LOCAL,
    behavior: { preflight: PreflightBehavior.optionalSuccess(token()) },
  },
  expected: WorkerFetchTestResult.SUCCESS,
}), "treat-as-public to local: success.");

promise_test(t => workerFetchTest(t, {
  source: {
    server: Server.HTTPS_LOCAL,
    treatAsPublic: true,
  },
  target: {
    server: Server.HTTPS_PRIVATE,
    behavior: { response: ResponseBehavior.allowCrossOrigin() },
  },
  expected: WorkerFetchTestResult.FAILURE,
}), "treat-as-public to private: failed preflight.");

promise_test(t => workerFetchTest(t, {
  source: {
    server: Server.HTTPS_LOCAL,
    treatAsPublic: true,
  },
  target: {
    server: Server.HTTPS_PRIVATE,
    behavior: {
      preflight: PreflightBehavior.success(token()),
      response: ResponseBehavior.allowCrossOrigin(),
    },
  },
  expected: WorkerFetchTestResult.SUCCESS,
}), "treat-as-public to private: success.");

promise_test(t => workerFetchTest(t, {
  source: {
    server: Server.HTTPS_LOCAL,
    treatAsPublic: true,
  },
  target: {
    server: Server.HTTPS_PUBLIC,
    behavior: { response: ResponseBehavior.allowCrossOrigin() },
  },
  expected: WorkerFetchTestResult.SUCCESS,
}), "treat-as-public to public: success.");
