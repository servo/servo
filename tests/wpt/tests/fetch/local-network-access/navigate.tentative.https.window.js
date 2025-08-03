// META: script=/common/subset-tests-by-key.js
// META: script=/common/dispatcher/dispatcher.js
// META: script=/common/utils.js
// META: script=resources/support.sub.js
// META: timeout=long
// META: variant=?include=from-loopback
// META: variant=?include=from-local
// META: variant=?include=from-public
// META: variant=?include=from-treat-as-public
//
// These tests verify that secure contexts can make top-level navigations
// to less-public address spaces. These are not restricted under LNA.

setup(() => {
  assert_true(window.isSecureContext);
});

// Source: secure loopback context.
//
// All top-level navigations unaffected by Local Network Access.

subsetTestByKey(
    'from-loopback', promise_test, t => navigateTest(t, {
                                     source: Server.HTTPS_LOOPBACK,
                                     target: Server.HTTPS_LOOPBACK,
                                     expected: NavigationTestResult.SUCCESS,
                                   }),
    'loopback to loopback: no permission required.');

subsetTestByKey(
    'from-loopback', promise_test, t => navigateTest(t, {
                                     source: Server.HTTPS_LOOPBACK,
                                     target: Server.HTTPS_LOCAL,
                                     expected: NavigationTestResult.SUCCESS,
                                   }),
    'loopback to local: no permission required.');

subsetTestByKey(
    'from-loopback', promise_test, t => navigateTest(t, {
                                     source: Server.HTTPS_LOOPBACK,
                                     target: Server.HTTPS_PUBLIC,
                                     expected: NavigationTestResult.SUCCESS,
                                   }),
    'loopback to public: no preflight required.');

// Source: secure local context.
//
// All top-level navigations unaffected by Local Network Access.

subsetTestByKey(
    'from-local', promise_test, t => navigateTest(t, {
                                  source: Server.HTTPS_LOCAL,
                                  target: Server.HTTPS_LOOPBACK,
                                  expected: NavigationTestResult.SUCCESS,
                                }),
    'local to loopback: no permission required.');

subsetTestByKey(
    'from-local', promise_test, t => navigateTest(t, {
                                  source: Server.HTTPS_LOCAL,
                                  target: Server.HTTPS_LOCAL,
                                  expected: NavigationTestResult.SUCCESS,
                                }),
    'local to local: no permission required.');

subsetTestByKey(
    'from-local', promise_test, t => navigateTest(t, {
                                  source: Server.HTTPS_LOCAL,
                                  target: Server.HTTPS_PUBLIC,
                                  expected: NavigationTestResult.SUCCESS,
                                }),
    'local to public: no permission required.');

// Source: secure public context.
//
// All top-level navigations unaffected by Local Network Access

subsetTestByKey(
    'from-public', promise_test, t => navigateTest(t, {
                                   source: Server.HTTPS_PUBLIC,
                                   target: Server.HTTPS_LOOPBACK,
                                   expected: NavigationTestResult.SUCCESS,
                                 }),
    'public to loopback: no permission required.');

subsetTestByKey(
    'from-public', promise_test, t => navigateTest(t, {
                                   source: Server.HTTPS_PUBLIC,
                                   target: Server.HTTPS_LOCAL,
                                   expected: NavigationTestResult.SUCCESS,
                                 }),
    'public to local: no permission required.');

subsetTestByKey(
    'from-public', promise_test, t => navigateTest(t, {
                                   source: Server.HTTPS_PUBLIC,
                                   target: Server.HTTPS_PUBLIC,
                                   expected: NavigationTestResult.SUCCESS,
                                 }),
    'public to public: no permission required.');

// The following tests verify that `CSP: treat-as-public-address` makes
// documents behave as if they had been served from a public IP address.

subsetTestByKey(
    'from-treat-as-public', promise_test,
    t => navigateTest(t, {
      source: {
        server: Server.HTTPS_LOOPBACK,
        treatAsPublic: true,
      },
      target: Server.OTHER_HTTPS_LOOPBACK,
      expected: NavigationTestResult.SUCCESS,
    }),
    'treat-as-public-address to loopback: no permission required.');

subsetTestByKey(
    'from-treat-as-public', promise_test,
    t => navigateTest(t, {
      source: {
        server: Server.HTTPS_LOOPBACK,
        treatAsPublic: true,
      },
      target: Server.HTTPS_LOOPBACK,
      expected: NavigationTestResult.SUCCESS,
    }),
    'treat-as-public-address to loopback (same-origin): no permission required.');

subsetTestByKey(
    'from-treat-as-public', promise_test,
    t => navigateTest(t, {
      source: {
        server: Server.HTTPS_LOOPBACK,
        treatAsPublic: true,
      },
      target: Server.HTTPS_LOCAL,
      expected: NavigationTestResult.SUCCESS,
    }),
    'treat-as-public-address to local: no permission required.');

subsetTestByKey(
    'from-treat-as-public', promise_test,
    t => navigateTest(t, {
      source: {
        server: Server.HTTPS_LOOPBACK,
        treatAsPublic: true,
      },
      target: Server.HTTPS_PUBLIC,
      expected: NavigationTestResult.SUCCESS,
    }),
    'treat-as-public-address to public: no permission required.');
