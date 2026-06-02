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
// These tests verify that nonsecure contexts can make top-level navigations
// to less-public address spaces. These are not restricted under LNA.
//
// This file covers only those tests that must execute in a non secure context.
// Other tests are defined in: navigate.tentative.https.window.js

setup(() => {
  assert_false(window.isSecureContext);
});

// Source: nonsecure loopback context.
//
// All top-level navigations unaffected by Local Network Access.

subsetTestByKey(
    'from-loopback', promise_test, t => navigateTest(t, {
                                     source: Server.HTTP_LOOPBACK,
                                     target: Server.HTTP_LOOPBACK,
                                     expected: NavigationTestResult.SUCCESS,
                                   }),
    'loopback to loopback: no permission required.');

subsetTestByKey(
    'from-loopback', promise_test, t => navigateTest(t, {
                                     source: Server.HTTP_LOOPBACK,
                                     target: Server.HTTP_LOCAL,
                                     expected: NavigationTestResult.SUCCESS,
                                   }),
    'loopback to local: no permission required.');

subsetTestByKey(
    'from-loopback', promise_test, t => navigateTest(t, {
                                     source: Server.HTTP_LOOPBACK,
                                     target: Server.HTTP_PUBLIC,
                                     expected: NavigationTestResult.SUCCESS,
                                   }),
    'loopback to public: no preflight required.');

// Source: secure local context.
//
// All top-level navigations unaffected by Local Network Access.

subsetTestByKey(
    'from-local', promise_test, t => navigateTest(t, {
                                  source: Server.HTTP_LOCAL,
                                  target: Server.HTTP_LOOPBACK,
                                  expected: NavigationTestResult.SUCCESS,
                                }),
    'local to loopback: no permission required.');

subsetTestByKey(
    'from-local', promise_test, t => navigateTest(t, {
                                  source: Server.HTTP_LOCAL,
                                  target: Server.HTTP_LOCAL,
                                  expected: NavigationTestResult.SUCCESS,
                                }),
    'local to local: no permission required.');

subsetTestByKey(
    'from-local', promise_test, t => navigateTest(t, {
                                  source: Server.HTTP_LOCAL,
                                  target: Server.HTTP_PUBLIC,
                                  expected: NavigationTestResult.SUCCESS,
                                }),
    'local to public: no permission required.');

// Source: secure public context.
//
// All top-level navigations unaffected by Local Network Access

subsetTestByKey(
    'from-public', promise_test, t => navigateTest(t, {
                                   source: Server.HTTP_PUBLIC,
                                   target: Server.HTTP_LOOPBACK,
                                   expected: NavigationTestResult.SUCCESS,
                                 }),
    'public to loopback: no permission required.');

subsetTestByKey(
    'from-public', promise_test, t => navigateTest(t, {
                                   source: Server.HTTP_PUBLIC,
                                   target: Server.HTTP_LOCAL,
                                   expected: NavigationTestResult.SUCCESS,
                                 }),
    'public to local: no permission required.');

subsetTestByKey(
    'from-public', promise_test, t => navigateTest(t, {
                                   source: Server.HTTP_PUBLIC,
                                   target: Server.HTTP_PUBLIC,
                                   expected: NavigationTestResult.SUCCESS,
                                 }),
    'public to public: no permission required.');

// The following tests verify that `CSP: treat-as-public-address` makes
// documents behave as if they had been served from a public IP address.

subsetTestByKey(
    'from-treat-as-public', promise_test,
    t => navigateTest(t, {
      source: {
        server: Server.HTTP_LOOPBACK,
        treatAsPublic: true,
      },
      target: Server.OTHER_HTTP_LOOPBACK,
      expected: NavigationTestResult.SUCCESS,
    }),
    'treat-as-public-address to loopback: no permission required.');

subsetTestByKey(
    'from-treat-as-public', promise_test,
    t => navigateTest(t, {
      source: {
        server: Server.HTTP_LOOPBACK,
        treatAsPublic: true,
      },
      target: Server.HTTP_LOOPBACK,
      expected: NavigationTestResult.SUCCESS,
    }),
    'treat-as-public-address to loopback (same-origin): no permission required.');

subsetTestByKey(
    'from-treat-as-public', promise_test,
    t => navigateTest(t, {
      source: {
        server: Server.HTTP_LOOPBACK,
        treatAsPublic: true,
      },
      target: Server.HTTP_LOCAL,
      expected: NavigationTestResult.SUCCESS,
    }),
    'treat-as-public-address to local: no permission required.');

subsetTestByKey(
    'from-treat-as-public', promise_test,
    t => navigateTest(t, {
      source: {
        server: Server.HTTP_LOOPBACK,
        treatAsPublic: true,
      },
      target: Server.HTTP_PUBLIC,
      expected: NavigationTestResult.SUCCESS,
    }),
    'treat-as-public-address to public: no permission required.');
