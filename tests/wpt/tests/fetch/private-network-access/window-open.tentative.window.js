// META: script=/common/dispatcher/dispatcher.js
// META: script=/common/utils.js
// META: script=resources/support.sub.js
//
// Spec: https://wicg.github.io/private-network-access/
//
// These tests verify that non-secure contexts cannot open a new window to
// less-public address spaces.

setup(() => {
  // Making sure we are in a non secure context, as expected.
  assert_false(window.isSecureContext);
});

promise_test(t => windowOpenTest(t, {
  source: { server: Server.HTTP_LOCAL },
  target: { server: Server.HTTP_LOCAL },
  expected: WindowOpenTestResult.SUCCESS,
}), "local to local: no preflight required.");

promise_test_parallel(t => windowOpenTest(t, {
  source: { server: Server.HTTP_LOCAL },
  target: { server: Server.HTTP_PRIVATE },
  expected: WindowOpenTestResult.SUCCESS,
}), "local to private: no preflight required.");

promise_test_parallel(t => windowOpenTest(t, {
  source: { server: Server.HTTP_LOCAL },
  target: { server: Server.HTTP_PUBLIC },
  expected: WindowOpenTestResult.SUCCESS,
}), "local to public: no preflight required.");

promise_test_parallel(t => windowOpenTest(t, {
  source: { server: Server.HTTP_PRIVATE },
  target: { server: Server.HTTP_LOCAL },
  expected: WindowOpenTestResult.FAILURE,
}), "private to local: failure.");

promise_test_parallel(t => windowOpenTest(t, {
  source: { server: Server.HTTP_PRIVATE },
  target: { server: Server.HTTP_PRIVATE },
  expected: WindowOpenTestResult.SUCCESS,
}), "private to private: no preflight required.");

promise_test_parallel(t => windowOpenTest(t, {
  source: { server: Server.HTTP_PRIVATE },
  target: { server: Server.HTTP_PUBLIC },
  expected: WindowOpenTestResult.SUCCESS,
}), "private to public: no preflight required.");

promise_test_parallel(t => windowOpenTest(t, {
  source: { server: Server.HTTP_PUBLIC },
  target: { server: Server.HTTP_LOCAL },
  expected: WindowOpenTestResult.FAILURE,
}), "public to local: failure.");

promise_test_parallel(t => windowOpenTest(t, {
  source: { server: Server.HTTP_PUBLIC },
  target: { server: Server.HTTP_PRIVATE },
  expected: WindowOpenTestResult.FAILURE,
}), "public to private: failure.");

promise_test_parallel(t => windowOpenTest(t, {
  source: { server: Server.HTTP_PUBLIC },
  target: { server: Server.HTTP_PUBLIC },
  expected: WindowOpenTestResult.SUCCESS,
}), "public to public: no preflight required.");

promise_test_parallel(t => windowOpenTest(t, {
  source: {
    server: Server.HTTP_LOCAL,
    treatAsPublic: true,
  },
  target: { server: Server.HTTP_LOCAL },
  expected: WindowOpenTestResult.FAILURE,
}), "treat-as-public-address to local: failure.");

promise_test_parallel(t => windowOpenTest(t, {
  source: {
    server: Server.HTTP_LOCAL,
    treatAsPublic: true,
  },
  target: { server: Server.HTTP_PRIVATE },
  expected: WindowOpenTestResult.FAILURE,
}), "treat-as-public-address to private: failure.");

promise_test_parallel(t => windowOpenTest(t, {
  source: {
    server: Server.HTTP_LOCAL,
    treatAsPublic: true,
  },
  target: { server: Server.HTTP_PUBLIC },
  expected: WindowOpenTestResult.SUCCESS,
}), "treat-as-public-address to public: no preflight required.");
