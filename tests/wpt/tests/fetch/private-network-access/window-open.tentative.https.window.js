// META: script=/common/dispatcher/dispatcher.js
// META: script=/common/utils.js
// META: script=resources/support.sub.js
//
// These tests verify that secure contexts can navigate iframes to less-public
// address spaces iff the target server responds affirmatively to preflight
// requests.

setup(() => {
  assert_true(window.isSecureContext);
});

// Source: secure local context.
//
// All fetches unaffected by Private Network Access.

promise_test_parallel(t => windowOpenTest(t, {
  source: { server: Server.HTTPS_LOCAL },
  target: { server: Server.HTTPS_LOCAL },
  expected: WindowOpenTestResult.SUCCESS,
}), "local to local: no preflight required.");

promise_test_parallel(t => windowOpenTest(t, {
  source: { server: Server.HTTPS_LOCAL },
  target: { server: Server.HTTPS_PRIVATE },
  expected: WindowOpenTestResult.SUCCESS,
}), "local to private: no preflight required.");

promise_test_parallel(t => windowOpenTest(t, {
  source: { server: Server.HTTPS_LOCAL },
  target: { server: Server.HTTPS_PUBLIC },
  expected: WindowOpenTestResult.SUCCESS,
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

  promise_test_parallel(t => windowOpenTest(t, {
    source,
    target: {
      server: targetServer,
      behavior: { preflight: PreflightBehavior.failure() },
    },
    expected: WindowOpenTestResult.FAILURE,
  }), prefix + "failed preflight.");

  promise_test_parallel(t => windowOpenTest(t, {
    source,
    target: {
      server: targetServer,
      behavior: { preflight: PreflightBehavior.noCorsHeader(token()) },
    },
    expected: WindowOpenTestResult.FAILURE,
  }), prefix + "missing CORS headers.");

  promise_test_parallel(t => windowOpenTest(t, {
    source,
    target: {
      server: targetServer,
      behavior: { preflight: PreflightBehavior.noPnaHeader(token()) },
    },
    expected: WindowOpenTestResult.FAILURE,
  }), prefix + "missing PNA header.");

  promise_test_parallel(t => windowOpenTest(t, {
    source,
    target: {
      server: targetServer,
      behavior: { preflight: PreflightBehavior.success(token()) },
    },
    expected: WindowOpenTestResult.SUCCESS,
  }), prefix + "success.");
}

// Source: private secure context.
//
// Fetches to the local address space require a successful preflight response
// carrying a PNA-specific header.

makePreflightTests({
  sourceServer: Server.HTTPS_PRIVATE,
  sourceName: 'private',
  targetServer: Server.HTTPS_LOCAL,
  targetName: 'local',
});

promise_test_parallel(t => windowOpenTest(t, {
  source: { server: Server.HTTPS_PRIVATE },
  target: { server: Server.HTTPS_PRIVATE },
  expected: WindowOpenTestResult.SUCCESS,
}), "private to private: no preflight required.");

promise_test_parallel(t => windowOpenTest(t, {
  source: { server: Server.HTTPS_PRIVATE },
  target: { server: Server.HTTPS_PUBLIC },
  expected: WindowOpenTestResult.SUCCESS,
}), "private to public: no preflight required.");

// Source: public secure context.
//
// Fetches to the local and private address spaces require a successful
// preflight response carrying a PNA-specific header.

makePreflightTests({
  sourceServer: Server.HTTPS_PUBLIC,
  sourceName: "public",
  targetServer: Server.HTTPS_LOCAL,
  targetName: "local",
});

makePreflightTests({
  sourceServer: Server.HTTPS_PUBLIC,
  sourceName: "public",
  targetServer: Server.HTTPS_PRIVATE,
  targetName: "private",
});

promise_test_parallel(t => windowOpenTest(t, {
  source: { server: Server.HTTPS_PUBLIC },
  target: { server: Server.HTTPS_PUBLIC },
  expected: WindowOpenTestResult.SUCCESS,
}), "public to public: no preflight required.");

// The following tests verify that `CSP: treat-as-public-address` makes
// documents behave as if they had been served from a public IP address.

makePreflightTests({
  sourceServer: Server.HTTPS_LOCAL,
  sourceTreatAsPublic: true,
  sourceName: "treat-as-public-address",
  targetServer: Server.OTHER_HTTPS_LOCAL,
  targetName: "local",
});

promise_test_parallel(
    t => windowOpenTest(t, {
      source: {
        server: Server.HTTPS_LOCAL,
        treatAsPublic: true,
      },
      target: {server: Server.HTTPS_LOCAL},
      expected: WindowOpenTestResult.SUCCESS,
    }),
    'treat-as-public-address to local (same-origin): no preflight required.');

makePreflightTests({
  sourceServer: Server.HTTPS_LOCAL,
  sourceTreatAsPublic: true,
  sourceName: 'treat-as-public-address',
  targetServer: Server.HTTPS_PRIVATE,
  targetName: 'private',
});

promise_test_parallel(
    t => windowOpenTest(t, {
      source: {
        server: Server.HTTPS_LOCAL,
        treatAsPublic: true,
      },
      target: {server: Server.HTTPS_PUBLIC},
      expected: WindowOpenTestResult.SUCCESS,
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
      expected: WindowOpenTestResult.SUCCESS,
    }),
    'treat-as-public-address to local: optional preflight');
