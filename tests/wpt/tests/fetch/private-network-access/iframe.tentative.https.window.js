// META: script=/common/subset-tests-by-key.js
// META: script=/common/dispatcher/dispatcher.js
// META: script=/common/utils.js
// META: script=resources/support.sub.js
// META: timeout=long
// META: variant=?include=from-local
// META: variant=?include=from-private
// META: variant=?include=from-public
// META: variant=?include=from-treat-as-public
// META: variant=?include=grandparent
//
// Spec: https://wicg.github.io/private-network-access/#integration-fetch
//
// These tests verify that secure contexts can navigate iframes to less-public
// address spaces iff the target server responds affirmatively to preflight
// requests.
//
// This file covers only those tests that must execute in a secure context.
// Other tests are defined in: iframe.tentative.window.js

setup(() => {
  assert_true(window.isSecureContext);
});

// Source: secure local context.
//
// All fetches unaffected by Private Network Access.

subsetTestByKey("from-local", promise_test_parallel, t => iframeTest(t, {
  source: { server: Server.HTTPS_LOCAL },
  target: { server: Server.HTTPS_LOCAL },
  expected: FrameTestResult.SUCCESS,
}), "local to local: no preflight required.");

subsetTestByKey("from-local", promise_test_parallel, t => iframeTest(t, {
  source: { server: Server.HTTPS_LOCAL },
  target: { server: Server.HTTPS_PRIVATE },
  expected: FrameTestResult.SUCCESS,
}), "local to private: no preflight required.");

subsetTestByKey("from-local", promise_test_parallel, t => iframeTest(t, {
  source: { server: Server.HTTPS_LOCAL },
  target: { server: Server.HTTPS_PUBLIC },
  expected: FrameTestResult.SUCCESS,
}), "local to public: no preflight required.");

// Generates tests of preflight behavior for a single (source, target) pair.
//
// Scenarios:
//
// - parent navigates child:
//   - preflight response has non-2xx HTTP code
//   - preflight response is missing CORS headers
//   - preflight response is missing the PNA-specific `Access-Control` header
//   - success
//
function makePreflightTests({
  key,
  sourceName,
  sourceServer,
  sourceTreatAsPublic,
  targetName,
  targetServer,
}) {
  const prefix =
      `${sourceName} to ${targetName}: `;

  const source = {
    server: sourceServer,
    treatAsPublic: sourceTreatAsPublic,
  };

  promise_test_parallel(t => iframeTest(t, {
    source,
    target: {
      server: targetServer,
      behavior: { preflight: PreflightBehavior.failure() },
    },
    expected: FrameTestResult.FAILURE,
  }), prefix + "failed preflight.");

  promise_test_parallel(t => iframeTest(t, {
    source,
    target: {
      server: targetServer,
      behavior: { preflight: PreflightBehavior.noCorsHeader(token()) },
    },
    expected: FrameTestResult.FAILURE,
  }), prefix + "missing CORS headers.");

  promise_test_parallel(t => iframeTest(t, {
    source,
    target: {
      server: targetServer,
      behavior: { preflight: PreflightBehavior.noPnaHeader(token()) },
    },
    expected: FrameTestResult.FAILURE,
  }), prefix + "missing PNA header.");

  promise_test_parallel(t => iframeTest(t, {
    source,
    target: {
      server: targetServer,
      behavior: { preflight: PreflightBehavior.navigation(token()) },
    },
    expected: FrameTestResult.SUCCESS,
  }), prefix + "success.");
}

// Source: private secure context.
//
// Fetches to the local address space require a successful preflight response
// carrying a PNA-specific header.

subsetTestByKey('from-private', makePreflightTests, {
  sourceServer: Server.HTTPS_PRIVATE,
  sourceName: 'private',
  targetServer: Server.HTTPS_LOCAL,
  targetName: 'local',
});

subsetTestByKey("from-private", promise_test_parallel, t => iframeTest(t, {
  source: { server: Server.HTTPS_PRIVATE },
  target: { server: Server.HTTPS_PRIVATE },
  expected: FrameTestResult.SUCCESS,
}), "private to private: no preflight required.");

subsetTestByKey("from-private", promise_test_parallel, t => iframeTest(t, {
  source: { server: Server.HTTPS_PRIVATE },
  target: { server: Server.HTTPS_PUBLIC },
  expected: FrameTestResult.SUCCESS,
}), "private to public: no preflight required.");

// Source: public secure context.
//
// Fetches to the local and private address spaces require a successful
// preflight response carrying a PNA-specific header.

subsetTestByKey('from-public', makePreflightTests, {
  sourceServer: Server.HTTPS_PUBLIC,
  sourceName: "public",
  targetServer: Server.HTTPS_LOCAL,
  targetName: "local",
});

subsetTestByKey('from-public', makePreflightTests, {
  sourceServer: Server.HTTPS_PUBLIC,
  sourceName: "public",
  targetServer: Server.HTTPS_PRIVATE,
  targetName: "private",
});

subsetTestByKey("from-public", promise_test_parallel, t => iframeTest(t, {
  source: { server: Server.HTTPS_PUBLIC },
  target: { server: Server.HTTPS_PUBLIC },
  expected: FrameTestResult.SUCCESS,
}), "public to public: no preflight required.");

// The following tests verify that `CSP: treat-as-public-address` makes
// documents behave as if they had been served from a public IP address.

subsetTestByKey('from-treat-as-public', makePreflightTests, {
  sourceServer: Server.HTTPS_LOCAL,
  sourceTreatAsPublic: true,
  sourceName: "treat-as-public-address",
  targetServer: Server.OTHER_HTTPS_LOCAL,
  targetName: "local",
});

subsetTestByKey(
  'from-treat-as-public', promise_test_parallel,
  t => iframeTest(t, {
    source: {
      server: Server.HTTPS_LOCAL,
      treatAsPublic: true,
    },
    target: {server: Server.HTTPS_LOCAL},
    expected: FrameTestResult.SUCCESS,
  }),
  'treat-as-public-address to local (same-origin): no preflight required.'
);

subsetTestByKey('from-treat-as-public', makePreflightTests, {
  sourceServer: Server.HTTPS_LOCAL,
  sourceTreatAsPublic: true,
  sourceName: "treat-as-public-address",
  targetServer: Server.HTTPS_PRIVATE,
  targetName: "private",
});

subsetTestByKey(
  'from-treat-as-public', promise_test_parallel,
  t => iframeTest(t, {
    source: {
      server: Server.HTTPS_LOCAL,
      treatAsPublic: true,
    },
    target: {server: Server.HTTPS_PUBLIC},
    expected: FrameTestResult.SUCCESS,
  }),
  'treat-as-public-address to public: no preflight required.'
);

subsetTestByKey(
  'from-treat-as-public', promise_test_parallel,
  t => iframeTest(t, {
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
  'treat-as-public-address to local: optional preflight'
);

// The following tests verify that when a grandparent frame navigates its
// grandchild, the IP address space of the grandparent is compared against the
// IP address space of the response. Indeed, the navigation initiator in this
// case is the grandparent, not the parent.

subsetTestByKey('grandparent', iframeGrandparentTest, {
  name: 'local to local, grandparent navigates: no preflight required.',
  grandparentServer: Server.HTTPS_LOCAL,
  child: {server: Server.HTTPS_PUBLIC},
  grandchild: {server: Server.OTHER_HTTPS_LOCAL},
  expected: FrameTestResult.SUCCESS,
});

subsetTestByKey('grandparent', iframeGrandparentTest, {
  name: "local to local (same-origin), grandparent navigates: no preflight required.",
  grandparentServer: Server.HTTPS_LOCAL,
  child: { server: Server.HTTPS_PUBLIC },
  grandchild: { server: Server.HTTPS_LOCAL },
  expected: FrameTestResult.SUCCESS,
});

subsetTestByKey('grandparent', iframeGrandparentTest, {
  name: "public to local, grandparent navigates: failure.",
  grandparentServer: Server.HTTPS_PUBLIC,
  child: {
    server: Server.HTTPS_LOCAL,
    behavior: { preflight: PreflightBehavior.navigation(token()) },
  },
  grandchild: {
    server: Server.HTTPS_LOCAL,
    behavior: { preflight: PreflightBehavior.failure() },
  },
  expected: FrameTestResult.FAILURE,
});

subsetTestByKey('grandparent', iframeGrandparentTest, {
  name: "public to local, grandparent navigates: success.",
  grandparentServer: Server.HTTPS_PUBLIC,
  child: {
    server: Server.HTTPS_LOCAL,
    behavior: { preflight: PreflightBehavior.navigation(token()) },
  },
  grandchild: {
    server: Server.HTTPS_LOCAL,
    behavior: { preflight: PreflightBehavior.navigation(token()) },
  },
  expected: FrameTestResult.SUCCESS,
});
