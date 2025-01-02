// META: script=/common/utils.js
// META: script=resources/support.sub.js
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

promise_test(t => makeServiceWorkerTest(t, {
  source: { server: Server.HTTPS_LOCAL },
  target: { server: Server.HTTPS_LOCAL },
  expected: TestResult.SUCCESS,
  fetch_document: true,
}), "local to local: success.");

promise_test(t => makeServiceWorkerTest(t, {
  source: { server: Server.HTTPS_PRIVATE },
  target: {
    server: Server.HTTPS_LOCAL,
    behavior: {
      preflight: PreflightBehavior.failure(),
      response: ResponseBehavior.allowCrossOrigin()
    },
  },
  expected: TestResult.FAILURE,
  fetch_document: true,
}), "private to local: failed preflight.");

promise_test(t => makeServiceWorkerTest(t, {
  source: { server: Server.HTTPS_PRIVATE },
  target: {
    server: Server.HTTPS_LOCAL,
    behavior: {
      preflight: PreflightBehavior.success(token()),
      response: ResponseBehavior.allowCrossOrigin(),
    },
  },
  expected: TestResult.SUCCESS,
  fetch_document: true,
}), "private to local: success.");

promise_test(t => makeServiceWorkerTest(t, {
  source: { server: Server.HTTPS_PRIVATE },
  target: { server: Server.HTTPS_PRIVATE },
  expected: TestResult.SUCCESS,
  fetch_document: true,
}), "private to private: success.");

promise_test(t => makeServiceWorkerTest(t, {
  source: { server: Server.HTTPS_PUBLIC },
  target: {
    server: Server.HTTPS_LOCAL,
    behavior: {
      preflight: PreflightBehavior.failure(),
      response: ResponseBehavior.allowCrossOrigin()
    },
  },
  expected: TestResult.FAILURE,
  fetch_document: true,
}), "public to local: failed preflight.");

promise_test(t => makeServiceWorkerTest(t, {
  source: { server: Server.HTTPS_PUBLIC },
  target: {
    server: Server.HTTPS_LOCAL,
    behavior: {
      preflight: PreflightBehavior.success(token()),
      response: ResponseBehavior.allowCrossOrigin(),
    },
  },
  expected: TestResult.SUCCESS,
  fetch_document: true,
}), "public to local: success.");

promise_test(t => makeServiceWorkerTest(t, {
  source: { server: Server.HTTPS_PUBLIC },
  target: {
    server: Server.HTTPS_PRIVATE,
    behavior: {
      preflight: PreflightBehavior.failure(),
      response: ResponseBehavior.allowCrossOrigin()
    },
  },
  expected: TestResult.FAILURE,
  fetch_document: true,
}), "public to private: failed preflight.");

promise_test(t => makeServiceWorkerTest(t, {
  source: { server: Server.HTTPS_PUBLIC },
  target: {
    server: Server.HTTPS_PRIVATE,
    behavior: {
      preflight: PreflightBehavior.success(token()),
      response: ResponseBehavior.allowCrossOrigin(),
    },
  },
  expected: TestResult.SUCCESS,
  fetch_document: true,
}), "public to private: success.");

promise_test(t => makeServiceWorkerTest(t, {
  source: { server: Server.HTTPS_PUBLIC },
  target: { server: Server.HTTPS_PUBLIC },
  expected: TestResult.SUCCESS,
  fetch_document: true,
}), "public to public: success.");

