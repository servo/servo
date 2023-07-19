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

// treat-as-public -> local -> private

// Request 1 (treat-as-public -> local): preflight required.
// Request 2 (treat-as-public -> private): preflight required.

// This verifies that PNA checks are applied to every step in a redirect chain.

promise_test(t => fetchTest(t, {
  source: {
    server: Server.HTTPS_LOCAL,
    treatAsPublic: true,
  },
  target: {
    server: Server.OTHER_HTTPS_LOCAL,
    behavior: {
      redirect: preflightUrl({
        server: Server.HTTPS_PRIVATE,
        behavior: {
          preflight: PreflightBehavior.success(token()),
          response: ResponseBehavior.allowCrossOrigin(),
        },
      }),
      response: ResponseBehavior.allowCrossOrigin(),
    }
  },
  expected: FetchTestResult.FAILURE,
}), "treat-as-public to local to private: failed first preflight.");

promise_test(t => fetchTest(t, {
  source: {
    server: Server.HTTPS_LOCAL,
    treatAsPublic: true,
  },
  target: {
    server: Server.OTHER_HTTPS_LOCAL,
    behavior: {
      preflight: PreflightBehavior.success(token()),
      redirect: preflightUrl({
        server: Server.HTTPS_PRIVATE,
        behavior: {
          preflight: PreflightBehavior.noPnaHeader(token()),
          response: ResponseBehavior.allowCrossOrigin(),
        },
      }),
      response: ResponseBehavior.allowCrossOrigin(),
    }
  },
  expected: FetchTestResult.FAILURE,
}), "treat-as-public to local to private: failed second preflight.");

promise_test(t => fetchTest(t, {
  source: {
    server: Server.HTTPS_LOCAL,
    treatAsPublic: true,
  },
  target: {
    server: Server.OTHER_HTTPS_LOCAL,
    behavior: {
      preflight: PreflightBehavior.success(token()),
      redirect: preflightUrl({
        server: Server.HTTPS_PRIVATE,
        behavior: {
          preflight: PreflightBehavior.success(token()),
          response: ResponseBehavior.allowCrossOrigin(),
        },
      }),
      response: ResponseBehavior.allowCrossOrigin(),
    }
  },
  expected: FetchTestResult.SUCCESS,
}), "treat-as-public to local to private: success.");

promise_test(t => fetchTest(t, {
  source: {
    server: Server.HTTPS_LOCAL,
    treatAsPublic: true,
  },
  target: {
    server: Server.OTHER_HTTPS_LOCAL,
    behavior: {
      redirect: preflightUrl({
        server: Server.HTTPS_PRIVATE,
        behavior: { preflight: PreflightBehavior.success(token()) },
      }),
    }
  },
  fetchOptions: { mode: "no-cors" },
  expected: FetchTestResult.FAILURE,
}), "treat-as-public to local to private: no-cors failed first preflight.");

promise_test(t => fetchTest(t, {
  source: {
    server: Server.HTTPS_LOCAL,
    treatAsPublic: true,
  },
  target: {
    server: Server.OTHER_HTTPS_LOCAL,
    behavior: {
      preflight: PreflightBehavior.success(token()),
      redirect: preflightUrl({ server: Server.HTTPS_PRIVATE }),
    }
  },
  fetchOptions: { mode: "no-cors" },
  expected: FetchTestResult.FAILURE,
}), "treat-as-public to local to private: no-cors failed second preflight.");

promise_test(t => fetchTest(t, {
  source: {
    server: Server.HTTPS_LOCAL,
    treatAsPublic: true,
  },
  target: {
    server: Server.OTHER_HTTPS_LOCAL,
    behavior: {
      preflight: PreflightBehavior.success(token()),
      redirect: preflightUrl({
        server: Server.HTTPS_PRIVATE,
        behavior: { preflight: PreflightBehavior.success(token()) },
      }),
    }
  },
  fetchOptions: { mode: "no-cors" },
  expected: FetchTestResult.OPAQUE,
}), "treat-as-public to local to private: no-cors success.");

// treat-as-public -> local (same-origin) -> private

// Request 1 (treat-as-public -> local (same-origin)): no preflight required.
// Request 2 (treat-as-public -> private): preflight required.

// This verifies that PNA checks are applied only to the second step in a
// redirect chain if the first step is same-origin and the origin is potentially
// trustworthy.

promise_test(t => fetchTest(t, {
  source: {
    server: Server.HTTPS_LOCAL,
    treatAsPublic: true,
  },
  target: {
    server: Server.HTTPS_LOCAL,
    behavior: {
      redirect: preflightUrl({
        server: Server.HTTPS_PRIVATE,
        behavior: {
          preflight: PreflightBehavior.noPnaHeader(token()),
          response: ResponseBehavior.allowCrossOrigin(),
        },
      }),
    }
  },
  expected: FetchTestResult.FAILURE,
}), "treat-as-public to local (same-origin) to private: failed second preflight.");

promise_test(t => fetchTest(t, {
  source: {
    server: Server.HTTPS_LOCAL,
    treatAsPublic: true,
  },
  target: {
    server: Server.HTTPS_LOCAL,
    behavior: {
      redirect: preflightUrl({
        server: Server.HTTPS_PRIVATE,
        behavior: {
          preflight: PreflightBehavior.success(token()),
          response: ResponseBehavior.allowCrossOrigin(),
        },
      }),
    }
  },
  expected: FetchTestResult.SUCCESS,
}), "treat-as-public to local (same-origin) to private: success.");

promise_test(t => fetchTest(t, {
  source: {
    server: Server.HTTPS_LOCAL,
    treatAsPublic: true,
  },
  target: {
    server: Server.HTTPS_LOCAL,
    behavior: {
      redirect: preflightUrl({ server: Server.HTTPS_PRIVATE }),
    }
  },
  fetchOptions: { mode: "no-cors" },
  expected: FetchTestResult.FAILURE,
}), "treat-as-public to local (same-origin) to private: no-cors failed second preflight.");

promise_test(t => fetchTest(t, {
  source: {
    server: Server.HTTPS_LOCAL,
    treatAsPublic: true,
  },
  target: {
    server: Server.HTTPS_LOCAL,
    behavior: {
      redirect: preflightUrl({
        server: Server.HTTPS_PRIVATE,
        behavior: { preflight: PreflightBehavior.success(token()) },
      }),
    }
  },
  fetchOptions: { mode: "no-cors" },
  expected: FetchTestResult.OPAQUE,
}), "treat-as-public to local (same-origin) to private: no-cors success.");

// treat-as-public -> private -> local

// Request 1 (treat-as-public -> private): preflight required.
// Request 2 (treat-as-public -> local): preflight required.

// This verifies that PNA checks are applied to every step in a redirect chain.

promise_test(t => fetchTest(t, {
  source: {
    server: Server.HTTPS_LOCAL,
    treatAsPublic: true,
  },
  target: {
    server: Server.HTTPS_PRIVATE,
    behavior: {
      preflight: PreflightBehavior.noPnaHeader(token()),
      response: ResponseBehavior.allowCrossOrigin(),
      redirect: preflightUrl({
        server: Server.OTHER_HTTPS_LOCAL,
        behavior: {
          preflight: PreflightBehavior.success(token()),
          response: ResponseBehavior.allowCrossOrigin(),
        },
      }),
    }
  },
  expected: FetchTestResult.FAILURE,
}), "treat-as-public to private to local: failed first preflight.");

promise_test(t => fetchTest(t, {
  source: {
    server: Server.HTTPS_LOCAL,
    treatAsPublic: true,
  },
  target: {
    server: Server.HTTPS_PRIVATE,
    behavior: {
      preflight: PreflightBehavior.success(token()),
      response: ResponseBehavior.allowCrossOrigin(),
      redirect: preflightUrl({
        server: Server.OTHER_HTTPS_LOCAL,
        behavior: { response: ResponseBehavior.allowCrossOrigin() },
      }),
    }
  },
  expected: FetchTestResult.FAILURE,
}), "treat-as-public to private to local: failed second preflight.");

promise_test(t => fetchTest(t, {
  source: {
    server: Server.HTTPS_LOCAL,
    treatAsPublic: true,
  },
  target: {
    server: Server.HTTPS_PRIVATE,
    behavior: {
      preflight: PreflightBehavior.success(token()),
      response: ResponseBehavior.allowCrossOrigin(),
      redirect: preflightUrl({
        server: Server.OTHER_HTTPS_LOCAL,
        behavior: {
          preflight: PreflightBehavior.success(token()),
          response: ResponseBehavior.allowCrossOrigin(),
        },
      }),
    }
  },
  expected: FetchTestResult.SUCCESS,
}), "treat-as-public to private to local: success.");

promise_test(t => fetchTest(t, {
  source: {
    server: Server.HTTPS_LOCAL,
    treatAsPublic: true,
  },
  target: {
    server: Server.HTTPS_PRIVATE,
    behavior: {
      redirect: preflightUrl({
        server: Server.OTHER_HTTPS_LOCAL,
        behavior: { preflight: PreflightBehavior.success(token()) },
      }),
    }
  },
  fetchOptions: { mode: "no-cors" },
  expected: FetchTestResult.FAILURE,
}), "treat-as-public to private to local: no-cors failed first preflight.");

promise_test(t => fetchTest(t, {
  source: {
    server: Server.HTTPS_LOCAL,
    treatAsPublic: true,
  },
  target: {
    server: Server.HTTPS_PRIVATE,
    behavior: {
      preflight: PreflightBehavior.success(token()),
      redirect: preflightUrl({ server: Server.OTHER_HTTPS_LOCAL }),
    }
  },
  fetchOptions: { mode: "no-cors" },
  expected: FetchTestResult.FAILURE,
}), "treat-as-public to private to local: no-cors failed second preflight.");

promise_test(t => fetchTest(t, {
  source: {
    server: Server.HTTPS_LOCAL,
    treatAsPublic: true,
  },
  target: {
    server: Server.HTTPS_PRIVATE,
    behavior: {
      preflight: PreflightBehavior.success(token()),
      redirect: preflightUrl({
        server: Server.OTHER_HTTPS_LOCAL,
        behavior: { preflight: PreflightBehavior.success(token()) },
      }),
    }
  },
  fetchOptions: { mode: "no-cors" },
  expected: FetchTestResult.OPAQUE,
}), "treat-as-public to private to local: no-cors success.");

// treat-as-public -> private -> local (same-origin)

// Request 1 (treat-as-public -> private): preflight required.
// Request 2 (treat-as-public -> local (same-origin)): no preflight required.

// This verifies that PNA checks are only applied to the first step in a
// redirect chain if the second step is same-origin and the origin is
// potentially trustworthy.

promise_test(t => fetchTest(t, {
  source: {
    server: Server.HTTPS_LOCAL,
    treatAsPublic: true,
  },
  target: {
    server: Server.HTTPS_PRIVATE,
    behavior: {
      preflight: PreflightBehavior.noPnaHeader(token()),
      response: ResponseBehavior.allowCrossOrigin(),
      redirect: preflightUrl({
        server: Server.HTTPS_LOCAL,
        behavior: { response: ResponseBehavior.allowCrossOrigin() },
      }),
    }
  },
  expected: FetchTestResult.FAILURE,
}), "treat-as-public to private to local (same-origin): failed first preflight.");

promise_test(t => fetchTest(t, {
  source: {
    server: Server.HTTPS_LOCAL,
    treatAsPublic: true,
  },
  target: {
    server: Server.HTTPS_PRIVATE,
    behavior: {
      preflight: PreflightBehavior.success(token()),
      response: ResponseBehavior.allowCrossOrigin(),
      redirect: preflightUrl({
        server: Server.HTTPS_LOCAL,
        behavior: { response: ResponseBehavior.allowCrossOrigin() },
      }),
    }
  },
  expected: FetchTestResult.SUCCESS,
}), "treat-as-public to private to local (same-origin): success.");

promise_test(t => fetchTest(t, {
  source: {
    server: Server.HTTPS_LOCAL,
    treatAsPublic: true,
  },
  target: {
    server: Server.HTTPS_PRIVATE,
    behavior: {
      redirect: preflightUrl({ server: Server.HTTPS_LOCAL }),
    }
  },
  fetchOptions: { mode: "no-cors" },
  expected: FetchTestResult.FAILURE,
}), "treat-as-public to private to local (same-origin): no-cors failed first preflight.");

promise_test(t => fetchTest(t, {
  source: {
    server: Server.HTTPS_LOCAL,
    treatAsPublic: true,
  },
  target: {
    server: Server.HTTPS_PRIVATE,
    behavior: {
      preflight: PreflightBehavior.success(token()),
      redirect: preflightUrl({ server: Server.HTTPS_LOCAL }),
    }
  },
  fetchOptions: { mode: "no-cors" },
  expected: FetchTestResult.OPAQUE,
}), "treat-as-public to private to local (same-origin): no-cors success.");
