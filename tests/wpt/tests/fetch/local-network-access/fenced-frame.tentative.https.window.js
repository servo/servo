// META: script=/common/dispatcher/dispatcher.js
// META: script=/common/utils.js
// META: script=resources/support.sub.js
// META: script=/fenced-frame/resources/utils.js
// META: timeout=long
//
// Spec: https://wicg.github.io/private-network-access/#integration-fetch
//
// These tests verify that contexts can navigate fenced frames to less-public
// address spaces iff the target server responds affirmatively to preflight
// requests.

setup(() => {
  assert_true(window.isSecureContext);
});

// Source: secure local context.
//
// All fetches unaffected by Private Network Access.

promise_test_parallel(
    t => fencedFrameTest(t, {
      source: {server: Server.HTTPS_LOCAL},
      target: {server: Server.HTTPS_LOCAL},
      expected: FrameTestResult.SUCCESS,
    }),
    'local to local: no preflight required.');

promise_test_parallel(
    t => fencedFrameTest(t, {
      source: {server: Server.HTTPS_LOCAL},
      target: {server: Server.HTTPS_PRIVATE},
      expected: FrameTestResult.SUCCESS,
    }),
    'local to private: no preflight required.');

promise_test_parallel(
    t => fencedFrameTest(t, {
      source: {server: Server.HTTPS_LOCAL},
      target: {server: Server.HTTPS_PUBLIC},
      expected: FrameTestResult.SUCCESS,
    }),
    'local to public: no preflight required.');

// Generates tests of preflight behavior for a single (source, target) pair.
//
// Scenarios:
//
// - parent navigates child:
//   - preflight response has non-2xx HTTP code
//   - preflight response is missing CORS headers
//   - preflight response is missing the PNA-specific `Access-Control` header
//   - preflight response has the required PNA related headers, but still fails
//     because of the limitation of fenced frame that subjects to PNA checks.
//
function makePreflightTests({
  sourceName,
  sourceServer,
  sourceTreatAsPublic,
  targetName,
  targetServer,
}) {
  const prefix = `${sourceName} to ${targetName}: `;

  const source = {
    server: sourceServer,
    treatAsPublic: sourceTreatAsPublic,
  };

  promise_test_parallel(
      t => fencedFrameTest(t, {
        source,
        target: {
          server: targetServer,
          behavior: {preflight: PreflightBehavior.failure()},
        },
        expected: FrameTestResult.FAILURE,
      }),
      prefix + 'failed preflight.');

  promise_test_parallel(
      t => fencedFrameTest(t, {
        source,
        target: {
          server: targetServer,
          behavior: {preflight: PreflightBehavior.noCorsHeader(token())},
        },
        expected: FrameTestResult.FAILURE,
      }),
      prefix + 'missing CORS headers.');

  promise_test_parallel(
      t => fencedFrameTest(t, {
        source,
        target: {
          server: targetServer,
          behavior: {preflight: PreflightBehavior.noPnaHeader(token())},
        },
        expected: FrameTestResult.FAILURE,
      }),
      prefix + 'missing PNA header.');

  promise_test_parallel(
      t => fencedFrameTest(t, {
        source,
        target: {
          server: targetServer,
          behavior: {
            preflight: PreflightBehavior.success(token()),
            response: ResponseBehavior.allowCrossOrigin()
          },
        },
        expected: FrameTestResult.FAILURE,
      }),
      prefix + 'failed because fenced frames are incompatible with PNA.');
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

promise_test_parallel(
    t => fencedFrameTest(t, {
      source: {server: Server.HTTPS_PRIVATE},
      target: {server: Server.HTTPS_PRIVATE},
      expected: FrameTestResult.SUCCESS,
    }),
    'private to private: no preflight required.');

promise_test_parallel(
    t => fencedFrameTest(t, {
      source: {server: Server.HTTPS_PRIVATE},
      target: {server: Server.HTTPS_PUBLIC},
      expected: FrameTestResult.SUCCESS,
    }),
    'private to public: no preflight required.');

// Source: public secure context.
//
// Fetches to the local and private address spaces require a successful
// preflight response carrying a PNA-specific header.

makePreflightTests({
  sourceServer: Server.HTTPS_PUBLIC,
  sourceName: 'public',
  targetServer: Server.HTTPS_LOCAL,
  targetName: 'local',
});

makePreflightTests({
  sourceServer: Server.HTTPS_PUBLIC,
  sourceName: 'public',
  targetServer: Server.HTTPS_PRIVATE,
  targetName: 'private',
});

promise_test_parallel(
    t => fencedFrameTest(t, {
      source: {server: Server.HTTPS_PUBLIC},
      target: {server: Server.HTTPS_PUBLIC},
      expected: FrameTestResult.SUCCESS,
    }),
    'public to public: no preflight required.');

// The following tests verify that `CSP: treat-as-public-address` makes
// documents behave as if they had been served from a public IP address.

makePreflightTests({
  sourceServer: Server.HTTPS_LOCAL,
  sourceTreatAsPublic: true,
  sourceName: 'treat-as-public-address',
  targetServer: Server.OTHER_HTTPS_LOCAL,
  targetName: 'local',
});

promise_test_parallel(
    t => fencedFrameTest(t, {
      source: {
        server: Server.HTTPS_LOCAL,
        treatAsPublic: true,
      },
      target: {server: Server.HTTPS_LOCAL},
      expected: FrameTestResult.FAILURE,
    }),
    'treat-as-public-address to local (same-origin): fenced frame embedder ' +
    'initiated navigation has opaque origin.');

makePreflightTests({
  sourceServer: Server.HTTPS_LOCAL,
  sourceTreatAsPublic: true,
  sourceName: 'treat-as-public-address',
  targetServer: Server.HTTPS_PRIVATE,
  targetName: 'private',
});

promise_test_parallel(
    t => fencedFrameTest(t, {
      source: {
        server: Server.HTTPS_LOCAL,
        treatAsPublic: true,
      },
      target: {server: Server.HTTPS_PUBLIC},
      expected: FrameTestResult.SUCCESS,
    }),
    'treat-as-public-address to public: no preflight required.');

promise_test_parallel(
    t => fencedFrameTest(t, {
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
    'treat-as-public-address to local: optional preflight');
