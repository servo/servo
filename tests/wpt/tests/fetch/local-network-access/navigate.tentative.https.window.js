// META: script=/common/subset-tests-by-key.js
// META: script=/common/dispatcher/dispatcher.js
// META: script=/common/utils.js
// META: script=resources/support.sub.js
// META: timeout=long
// META: variant=?include=from-loopback
// META: variant=?include=from-local
// META: variant=?include=from-public
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
