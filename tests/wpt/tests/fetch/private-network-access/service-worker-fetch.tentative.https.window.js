// META: script=/common/utils.js
// META: script=resources/support.sub.js
// META: script=/common/subset-tests.js
// META: variant=?1-8
// META: variant=?9-last
//
// Spec: https://wicg.github.io/private-network-access/#integration-fetch
//
// These tests check that fetches from within `ServiceWorker` scripts are
// subject to Private Network Access checks, just like fetches from within
// documents.

// Results that may be expected in tests.
const TestResult = {
  SUCCESS: { ok: true, body: "success" },
  FAILURE: { error: "TypeError" },
};

subsetTest(promise_test, t => makeServiceWorkerTest(t, {
  source: { server: Server.HTTPS_LOCAL },
  target: { server: Server.HTTPS_LOCAL },
  expected: TestResult.SUCCESS,
}), "local to local: success.");

subsetTest(promise_test, t => makeServiceWorkerTest(t, {
  source: { server: Server.HTTPS_PRIVATE },
  target: {
    server: Server.HTTPS_LOCAL,
    behavior: {
      preflight: PreflightBehavior.failure(),
      response: ResponseBehavior.allowCrossOrigin()
    },
  },
  expected: TestResult.FAILURE,
}), "private to local: failed preflight.");

subsetTest(promise_test, t => makeServiceWorkerTest(t, {
  source: { server: Server.HTTPS_PRIVATE },
  target: {
    server: Server.HTTPS_LOCAL,
    behavior: {
      preflight: PreflightBehavior.success(token()),
      response: ResponseBehavior.allowCrossOrigin(),
    },
  },
  expected: TestResult.SUCCESS,
}), "private to local: success.");

subsetTest(promise_test, t => makeServiceWorkerTest(t, {
  source: { server: Server.HTTPS_PRIVATE },
  target: { server: Server.HTTPS_PRIVATE },
  expected: TestResult.SUCCESS,
}), "private to private: success.");

subsetTest(promise_test, t => makeServiceWorkerTest(t, {
  source: { server: Server.HTTPS_PUBLIC },
  target: {
    server: Server.HTTPS_LOCAL,
    behavior: {
      preflight: PreflightBehavior.failure(),
      response: ResponseBehavior.allowCrossOrigin()
    },
  },
  expected: TestResult.FAILURE,
}), "public to local: failed preflight.");

subsetTest(promise_test, t => makeServiceWorkerTest(t, {
  source: { server: Server.HTTPS_PUBLIC },
  target: {
    server: Server.HTTPS_LOCAL,
    behavior: {
      preflight: PreflightBehavior.success(token()),
      response: ResponseBehavior.allowCrossOrigin(),
    },
  },
  expected: TestResult.SUCCESS,
}), "public to local: success.");

subsetTest(promise_test, t => makeServiceWorkerTest(t, {
  source: { server: Server.HTTPS_PUBLIC },
  target: {
    server: Server.HTTPS_PRIVATE,
    behavior: {
      preflight: PreflightBehavior.failure(),
      response: ResponseBehavior.allowCrossOrigin()
    },
  },
  expected: TestResult.FAILURE,
}), "public to private: failed preflight.");

subsetTest(promise_test, t => makeServiceWorkerTest(t, {
  source: { server: Server.HTTPS_PUBLIC },
  target: {
    server: Server.HTTPS_PRIVATE,
    behavior: {
      preflight: PreflightBehavior.success(token()),
      response: ResponseBehavior.allowCrossOrigin(),
    },
  },
  expected: TestResult.SUCCESS,
}), "public to private: success.");

subsetTest(promise_test, t => makeServiceWorkerTest(t, {
  source: { server: Server.HTTPS_PUBLIC },
  target: { server: Server.HTTPS_PUBLIC },
  expected: TestResult.SUCCESS,
}), "public to public: success.");

subsetTest(promise_test, t => makeServiceWorkerTest(t, {
  source: {
    server: Server.HTTPS_LOCAL,
    treatAsPublic: true,
  },
  target: {
    server: Server.OTHER_HTTPS_LOCAL,
    behavior: {
      preflight: PreflightBehavior.failure(),
      response: ResponseBehavior.allowCrossOrigin()
    },
  },
  expected: TestResult.FAILURE,
}), "treat-as-public to local: failed preflight.");

subsetTest(promise_test, t => makeServiceWorkerTest(t, {
  source: {
    server: Server.HTTPS_LOCAL,
    treatAsPublic: true,
  },
  target: {
    server: Server.OTHER_HTTPS_LOCAL,
    behavior: {
      preflight: PreflightBehavior.success(token()),
      response: ResponseBehavior.allowCrossOrigin(),
    },
  },
  expected: TestResult.SUCCESS,
}), "treat-as-public to local: success.");

subsetTest(promise_test, t => makeServiceWorkerTest(t, {
  source: {
    server: Server.HTTPS_LOCAL,
    treatAsPublic: true,
  },
  target: { server: Server.HTTPS_LOCAL },
  expected: TestResult.SUCCESS,
}), "treat-as-public to local (same-origin): no preflight required.");

subsetTest(promise_test, t => makeServiceWorkerTest(t, {
  source: {
    server: Server.HTTPS_LOCAL,
    treatAsPublic: true,
  },
  target: {
    server: Server.HTTPS_PRIVATE,
    behavior: {
      preflight: PreflightBehavior.failure(),
      response: ResponseBehavior.allowCrossOrigin()
    },
  },
  expected: TestResult.FAILURE,
}), "treat-as-public to private: failed preflight.");

subsetTest(promise_test, t => makeServiceWorkerTest(t, {
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
  expected: TestResult.SUCCESS,
}), "treat-as-public to private: success.");

subsetTest(promise_test, t => makeServiceWorkerTest(t, {
  source: {
    server: Server.HTTPS_LOCAL,
    treatAsPublic: true,
  },
  target: {
    server: Server.HTTPS_PUBLIC,
    behavior: { response: ResponseBehavior.allowCrossOrigin() },
  },
  expected: TestResult.SUCCESS,
}), "treat-as-public to public: success.");
