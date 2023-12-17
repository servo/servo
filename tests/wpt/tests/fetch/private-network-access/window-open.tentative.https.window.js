// META: script=/common/subset-tests-by-key.js
// META: script=/common/dispatcher/dispatcher.js
// META: script=/common/utils.js
// META: script=resources/support.sub.js
// META: timeout=long
// META: variant=?include=from-local
// META: variant=?include=from-private
// META: variant=?include=from-public
// META: variant=?include=from-treat-as-public
//
// These tests verify that secure contexts can navigate to less-public address
// spaces via window.open iff the target server responds affirmatively to
// preflight requests.

setup(() => {
  assert_true(window.isSecureContext);
});

// Source: secure local context.
//
// All fetches unaffected by Private Network Access.

subsetTestByKey("from-local", promise_test_parallel, t => windowOpenTest(t, {
  source: { server: Server.HTTPS_LOCAL },
  target: { server: Server.HTTPS_LOCAL },
  expected: NavigationTestResult.SUCCESS,
}), "local to local: no preflight required.");

subsetTestByKey("from-local", promise_test_parallel, t => windowOpenTest(t, {
  source: { server: Server.HTTPS_LOCAL },
  target: { server: Server.HTTPS_PRIVATE },
  expected: NavigationTestResult.SUCCESS,
}), "local to private: no preflight required.");

subsetTestByKey("from-local", promise_test_parallel, t => windowOpenTest(t, {
  source: { server: Server.HTTPS_LOCAL },
  target: { server: Server.HTTPS_PUBLIC },
  expected: NavigationTestResult.SUCCESS,
}), "local to public: no preflight required.");

// Generates tests of preflight behavior for a single (source, target) pair.
//
// Scenarios:
//
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

  promise_test_parallel(t => windowOpenTest(t, {
    source,
    target: {
      server: targetServer,
      behavior: { preflight: PreflightBehavior.failure() },
    },
    expected: NavigationTestResult.FAILURE,
  }), prefix + "failed preflight.");

  promise_test_parallel(t => windowOpenTest(t, {
    source,
    target: {
      server: targetServer,
      behavior: { preflight: PreflightBehavior.noCorsHeader(token()) },
    },
    expected: NavigationTestResult.FAILURE,
  }), prefix + "missing CORS headers.");

  promise_test_parallel(t => windowOpenTest(t, {
    source,
    target: {
      server: targetServer,
      behavior: { preflight: PreflightBehavior.noPnaHeader(token()) },
    },
    expected: NavigationTestResult.FAILURE,
  }), prefix + "missing PNA header.");

  promise_test_parallel(t => windowOpenTest(t, {
    source,
    target: {
      server: targetServer,
      behavior: { preflight: PreflightBehavior.navigation(token()) },
    },
    expected: NavigationTestResult.SUCCESS,
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

subsetTestByKey("from-private", promise_test_parallel, t => windowOpenTest(t, {
  source: { server: Server.HTTPS_PRIVATE },
  target: { server: Server.HTTPS_PRIVATE },
  expected: NavigationTestResult.SUCCESS,
}), "private to private: no preflight required.");

subsetTestByKey("from-private", promise_test_parallel, t => windowOpenTest(t, {
  source: { server: Server.HTTPS_PRIVATE },
  target: { server: Server.HTTPS_PUBLIC },
  expected: NavigationTestResult.SUCCESS,
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

subsetTestByKey("from-public", promise_test_parallel, t => windowOpenTest(t, {
  source: { server: Server.HTTPS_PUBLIC },
  target: { server: Server.HTTPS_PUBLIC },
  expected: NavigationTestResult.SUCCESS,
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

subsetTestByKey("from-treat-as-public", promise_test_parallel,
    t => windowOpenTest(t, {
      source: {
        server: Server.HTTPS_LOCAL,
        treatAsPublic: true,
      },
      target: {server: Server.HTTPS_LOCAL},
      expected: NavigationTestResult.SUCCESS,
    }),
    'treat-as-public-address to local (same-origin): no preflight required.');

subsetTestByKey('from-treat-as-public', makePreflightTests, {
  sourceServer: Server.HTTPS_LOCAL,
  sourceTreatAsPublic: true,
  sourceName: 'treat-as-public-address',
  targetServer: Server.HTTPS_PRIVATE,
  targetName: 'private',
});

subsetTestByKey("from-treat-as-public", promise_test_parallel,
    t => windowOpenTest(t, {
      source: {
        server: Server.HTTPS_LOCAL,
        treatAsPublic: true,
      },
      target: {server: Server.HTTPS_PUBLIC},
      expected: NavigationTestResult.SUCCESS,
    }),
    'treat-as-public-address to public: no preflight required.');

promise_test_parallel(
    t => windowOpenTest(t, {
      source: {
        server: Server.HTTPS_LOCAL,
        treatAsPublic: true,
      },
      target: {
        server: Server.HTTPS_PUBLIC,
        behavior: {preflight: PreflightBehavior.optionalSuccess(token())}
      },
      expected: NavigationTestResult.SUCCESS,
    }),
    'treat-as-public-address to local: optional preflight');
