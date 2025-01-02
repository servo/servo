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
    fetch_document: true,
}), "treat-as-public to local: failed preflight.");

promise_test(t => makeServiceWorkerTest(t, {
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
    fetch_document: true,
}), "treat-as-public to local: success.");

promise_test(t => makeServiceWorkerTest(t, {
    source: {
      server: Server.HTTPS_LOCAL,
      treatAsPublic: true,
    },
    target: { server: Server.HTTPS_LOCAL },
    expected: TestResult.SUCCESS,
    fetch_document: true,
}), "treat-as-public to local (same-origin): no preflight required.");

promise_test(t => makeServiceWorkerTest(t, {
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
    fetch_document: true,
}), "treat-as-public to private: failed preflight.");

promise_test(t => makeServiceWorkerTest(t, {
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
    fetch_document: true,
}), "treat-as-public to private: success.");

promise_test(t => makeServiceWorkerTest(t, {
    source: {
      server: Server.HTTPS_LOCAL,
      treatAsPublic: true,
    },
    target: {
      server: Server.HTTPS_PUBLIC,
      behavior: { response: ResponseBehavior.allowCrossOrigin() },
    },
    expected: TestResult.SUCCESS,
    fetch_document: true,
}), "treat-as-public to public: success.");
