// META: script=/common/utils.js
// META: script=resources/support.sub.js
//
// Spec: https://wicg.github.io/private-network-access/#integration-fetch
//
// This test verifies that Private Network Access checks are applied to all
// the endpoints in a redirect chain, relative to the same client context.

// local -> private -> public
//
// Request 1 (local -> private): no preflight.
// Request 2 (local -> public): no preflight.

promise_test(t => fetchTest(t, {
  source: { server: Server.HTTPS_LOCAL },
  target: {
    server: Server.HTTPS_PRIVATE,
    behavior: {
      response: ResponseBehavior.allowCrossOrigin(),
      redirect: preflightUrl({
        server: Server.HTTPS_PUBLIC,
        behavior: { response: ResponseBehavior.allowCrossOrigin() },
      }),
    }
  },
  expected: FetchTestResult.SUCCESS,
}), "local to private to public: success.");

// local -> private -> local
//
// Request 1 (local -> private): no preflight.
// Request 2 (local -> local): no preflight.
//
// This checks that the client for the second request is still the initial
// context, not the redirector.

promise_test(t => fetchTest(t, {
  source: { server: Server.HTTPS_LOCAL },
  target: {
    server: Server.HTTPS_PRIVATE,
    behavior: {
      response: ResponseBehavior.allowCrossOrigin(),
      redirect: preflightUrl({
        server: Server.HTTPS_LOCAL,
        behavior: { response: ResponseBehavior.allowCrossOrigin() },
      }),
    }
  },
  expected: FetchTestResult.SUCCESS,
}), "local to private to local: success.");

// private -> private -> local
//
// Request 1 (private -> private): no preflight.
// Request 2 (private -> local): preflight required.
//
// This verifies that PNA checks are applied after redirects.

promise_test(t => fetchTest(t, {
  source: { server: Server.HTTPS_PRIVATE },
  target: {
    server: Server.HTTPS_PRIVATE,
    behavior: {
      redirect: preflightUrl({
        server: Server.HTTPS_LOCAL,
        behavior: { response: ResponseBehavior.allowCrossOrigin() },
      }),
    }
  },
  expected: FetchTestResult.FAILURE,
}), "private to private to local: failed preflight.");

promise_test(t => fetchTest(t, {
  source: { server: Server.HTTPS_PRIVATE },
  target: {
    server: Server.HTTPS_PRIVATE,
    behavior: {
      redirect: preflightUrl({
        server: Server.HTTPS_LOCAL,
        behavior: {
          preflight: PreflightBehavior.success(token()),
          response: ResponseBehavior.allowCrossOrigin(),
        },
      }),
    }
  },
  expected: FetchTestResult.SUCCESS,
}), "private to private to local: success.");

promise_test(t => fetchTest(t, {
  source: { server: Server.HTTPS_PRIVATE },
  target: {
    server: Server.HTTPS_PRIVATE,
    behavior: {
      redirect: preflightUrl({
        server: Server.HTTPS_LOCAL,
        behavior: { preflight: PreflightBehavior.success(token()) },
      }),
    }
  },
  fetchOptions: { mode: "no-cors" },
  expected: FetchTestResult.OPAQUE,
}), "private to private to local: no-cors success.");

// private -> local -> private
//
// Request 1 (private -> local): preflight required.
// Request 2 (private -> private): no preflight.
//
// This verifies that PNA checks are applied independently to every step in a
// redirect chain.

promise_test(t => fetchTest(t, {
  source: { server: Server.HTTPS_PRIVATE },
  target: {
    server: Server.HTTPS_LOCAL,
    behavior: {
      response: ResponseBehavior.allowCrossOrigin(),
      redirect: preflightUrl({
        server: Server.HTTPS_PRIVATE,
      }),
    }
  },
  expected: FetchTestResult.FAILURE,
}), "private to local to private: failed preflight.");

promise_test(t => fetchTest(t, {
  source: { server: Server.HTTPS_PRIVATE },
  target: {
    server: Server.HTTPS_LOCAL,
    behavior: {
      preflight: PreflightBehavior.success(token()),
      response: ResponseBehavior.allowCrossOrigin(),
      redirect: preflightUrl({
        server: Server.HTTPS_PRIVATE,
        behavior: { response: ResponseBehavior.allowCrossOrigin() },
      }),
    }
  },
  expected: FetchTestResult.SUCCESS,
}), "private to local to private: success.");

promise_test(t => fetchTest(t, {
  source: { server: Server.HTTPS_PRIVATE },
  target: {
    server: Server.HTTPS_LOCAL,
    behavior: {
      preflight: PreflightBehavior.success(token()),
      redirect: preflightUrl({ server: Server.HTTPS_PRIVATE }),
    }
  },
  fetchOptions: { mode: "no-cors" },
  expected: FetchTestResult.OPAQUE,
}), "private to local to private: no-cors success.");

// public -> private -> local
//
// Request 1 (public -> private): preflight required.
// Request 2 (public -> local): preflight required.
//
// This verifies that PNA checks are applied to every step in a redirect chain.

promise_test(t => fetchTest(t, {
  source: { server: Server.HTTPS_PUBLIC },
  target: {
    server: Server.HTTPS_PRIVATE,
    behavior: {
      response: ResponseBehavior.allowCrossOrigin(),
      redirect: preflightUrl({
        server: Server.HTTPS_LOCAL,
        behavior: {
          preflight: PreflightBehavior.success(token()),
          response: ResponseBehavior.allowCrossOrigin(),
        },
      }),
    }
  },
  expected: FetchTestResult.FAILURE,
}), "public to private to local: failed first preflight.");

promise_test(t => fetchTest(t, {
  source: { server: Server.HTTPS_PUBLIC },
  target: {
    server: Server.HTTPS_PRIVATE,
    behavior: {
      preflight: PreflightBehavior.success(token()),
      response: ResponseBehavior.allowCrossOrigin(),
      redirect: preflightUrl({
        server: Server.HTTPS_LOCAL,
        behavior: {
          response: ResponseBehavior.allowCrossOrigin(),
        },
      }),
    }
  },
  expected: FetchTestResult.FAILURE,
}), "public to private to local: failed second preflight.");

promise_test(t => fetchTest(t, {
  source: { server: Server.HTTPS_PUBLIC },
  target: {
    server: Server.HTTPS_PRIVATE,
    behavior: {
      preflight: PreflightBehavior.success(token()),
      response: ResponseBehavior.allowCrossOrigin(),
      redirect: preflightUrl({
        server: Server.HTTPS_LOCAL,
        behavior: {
          preflight: PreflightBehavior.success(token()),
          response: ResponseBehavior.allowCrossOrigin(),
        },
      }),
    }
  },
  expected: FetchTestResult.SUCCESS,
}), "public to private to local: success.");

promise_test(t => fetchTest(t, {
  source: { server: Server.HTTPS_PUBLIC },
  target: {
    server: Server.HTTPS_PRIVATE,
    behavior: {
      preflight: PreflightBehavior.success(token()),
      redirect: preflightUrl({
        server: Server.HTTPS_LOCAL,
        behavior: { preflight: PreflightBehavior.success(token()) },
      }),
    }
  },
  fetchOptions: { mode: "no-cors" },
  expected: FetchTestResult.OPAQUE,
}), "public to private to local: no-cors success.");
