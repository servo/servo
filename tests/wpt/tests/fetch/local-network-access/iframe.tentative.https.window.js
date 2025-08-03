// META: script=/common/subset-tests-by-key.js
// META: script=/common/dispatcher/dispatcher.js
// META: script=/common/utils.js
// META: script=/resources/testdriver.js
// META: script=/resources/testdriver-vendor.js
// META: script=resources/support.sub.js
// META: timeout=long
// META: variant=?include=from-loopback
// META: variant=?include=from-local
// META: variant=?include=from-public
// META: variant=?include=from-treat-as-public
//
// Spec: https://wicg.github.io/local-network-access/#integration-fetch
//
// These tests verify that secure contexts can navigate iframes to less-public
// address spaces iff the initiating document has been granted the LNA
// permission.
//
// This file covers only those tests that must execute in a secure context.

setup(() => {
  assert_true(window.isSecureContext);
});

// Source: secure loopback context.
//
// All iframe navigations unaffected by Local Network Access.

subsetTestByKey(
    'from-loopback', promise_test, t => iframeTest(t, {
                                     source: Server.HTTP_LOOPBACK,
                                     target: Server.HTTPS_LOOPBACK,
                                     expected: NavigationTestResult.SUCCESS,
                                   }),
    'loopback to loopback: no permission required.');

subsetTestByKey(
    'from-loopback', promise_test, t => iframeTest(t, {
                                     source: Server.HTTP_LOOPBACK,
                                     target: Server.HTTPS_LOCAL,
                                     expected: NavigationTestResult.SUCCESS,
                                   }),
    'loopback to local: no permission required.');

subsetTestByKey(
    'from-loopback', promise_test, t => iframeTest(t, {
                                     source: Server.HTTP_LOOPBACK,
                                     target: Server.HTTPS_PUBLIC,
                                     expected: NavigationTestResult.SUCCESS,
                                   }),
    'loopback to public: no permission required.');

// Source: local secure context.
//
// All iframe navigations unaffected by Local Network Access.

// Requests from the `local` address space to the `loopback` address space
// are not yet restricted by LNA.
subsetTestByKey(
    'from-local', promise_test, t => iframeTest(t, {
                                  source: Server.HTTP_LOCAL,
                                  target: Server.HTTPS_LOOPBACK,
                                  expected: NavigationTestResult.SUCCESS,
                                }),
    'local to loopback: no permission required.');

subsetTestByKey(
    'from-local', promise_test, t => iframeTest(t, {
                                  source: Server.HTTP_LOCAL,
                                  target: Server.HTTPS_LOCAL,
                                  expected: NavigationTestResult.SUCCESS,
                                }),
    'local to local: no permission required.');

subsetTestByKey(
    'from-local', promise_test, t => iframeTest(t, {
                                  source: Server.HTTP_LOCAL,
                                  target: Server.HTTPS_PUBLIC,
                                  expected: NavigationTestResult.SUCCESS,
                                }),
    'local to public: no permission required.');


// Generates tests of permission behavior for a single (source, target) pair.
//
// Scenarios:
//
// - parent (source) navigates child (target):
//   - parent has been denied the LNA permission (failure)
//   - parent has been granted the LNA permission (success)
//
function makePermissionTests({
  key,
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

  promise_test(
      t => iframeTest(t, {
        source,
        target: {
          server: targetServer,
        },
        expected: NavigationTestResult.FAILURE,
        permission: 'denied',
      }),
      prefix + 'permission denied.');

  promise_test(
      t => iframeTest(t, {
        source,
        target: {
          server: targetServer,
        },
        expected: NavigationTestResult.SUCCESS,
        permission: 'granted',
      }),
      prefix + 'success.');
}


// Source: public secure context.
//
// iframe navigations to the loopback and local address spaces require the LNA
// permission.

subsetTestByKey('from-public', makePermissionTests, {
  sourceServer: Server.HTTPS_PUBLIC,
  sourceName: 'public',
  targetServer: Server.HTTPS_LOOPBACK,
  targetName: 'loopback',
});

subsetTestByKey('from-public', makePermissionTests, {
  sourceServer: Server.HTTPS_PUBLIC,
  sourceName: 'public',
  targetServer: Server.HTTPS_LOCAL,
  targetName: 'local',
});

subsetTestByKey(
    'from-public', promise_test, t => iframeTest(t, {
                                   source: Server.HTTPS_PUBLIC,
                                   target: Server.HTTPS_PUBLIC,
                                   expected: NavigationTestResult.SUCCESS,
                                 }),
    'public to public: no permission required.');

// The following tests verify that `CSP: treat-as-public-address` makes
// documents behave as if they had been served from a public IP address.

subsetTestByKey('from-treat-as-public', makePermissionTests, {
  sourceServer: Server.HTTPS_LOOPBACK,
  sourceTreatAsPublic: true,
  sourceName: 'treat-as-public-address',
  targetServer: Server.OTHER_HTTPS_LOOPBACK,
  targetName: 'loopback',
});

subsetTestByKey(
    'from-treat-as-public', promise_test,
    t => iframeTest(t, {
      source: {
        server: Server.HTTPS_LOOPBACK,
        treatAsPublic: true,
      },
      target: Server.HTTPS_LOOPBACK,
      expected: NavigationTestResult.SUCCESS,
    }),
    'treat-as-public-address to local (same-origin): no permission required.');

subsetTestByKey('from-treat-as-public', makePermissionTests, {
  sourceServer: Server.HTTPS_LOOPBACK,
  sourceTreatAsPublic: true,
  sourceName: 'treat-as-public-address',
  targetServer: Server.HTTPS_LOCAL,
  targetName: 'local',
});

subsetTestByKey(
    'from-treat-as-public', promise_test,
    t => iframeTest(t, {
      source: {
        server: Server.HTTPS_LOOPBACK,
        treatAsPublic: true,
      },
      target: Server.HTTPS_PUBLIC,
      expected: NavigationTestResult.SUCCESS,
    }),
    'treat-as-public-address to public: no permission required.');
