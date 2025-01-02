// META: script=/common/utils.js
// META: script=resources/support.sub.js
//
// Spec: https://wicg.github.io/private-network-access/#cors-preflight
//
// These tests verify that PNA preflight responses are cached.
//
// TODO(https://crbug.com/1268312): We cannot currently test that cache
// entries are keyed by target IP address space because that requires
// loading the same URL from different IP address spaces, and the WPT
// framework does not allow that.
promise_test(async t => {
  let uuid = token();
  await fetchTest(t, {
    source: { server: Server.HTTPS_PRIVATE },
    target: {
      server: Server.HTTPS_LOCAL,
      behavior: {
        preflight: PreflightBehavior.singlePreflight(uuid),
        response: ResponseBehavior.allowCrossOrigin(),
      },
    },
    expected: FetchTestResult.SUCCESS,
  });
  await fetchTest(t, {
    source: { server: Server.HTTPS_PRIVATE },
    target: {
      server: Server.HTTPS_LOCAL,
      behavior: {
        preflight: PreflightBehavior.singlePreflight(uuid),
        response: ResponseBehavior.allowCrossOrigin(),
      },
    },
    expected: FetchTestResult.SUCCESS,
  });
}, "private to local: success.");

promise_test(async t => {
  let uuid = token();
  await fetchTest(t, {
    source: { server: Server.HTTPS_PUBLIC },
    target: {
      server: Server.HTTPS_LOCAL,
      behavior: {
        preflight: PreflightBehavior.singlePreflight(uuid),
        response: ResponseBehavior.allowCrossOrigin(),
      },
    },
    expected: FetchTestResult.SUCCESS,
  });
  await fetchTest(t, {
    source: { server: Server.HTTPS_PUBLIC },
    target: {
      server: Server.HTTPS_LOCAL,
      behavior: {
        preflight: PreflightBehavior.singlePreflight(uuid),
        response: ResponseBehavior.allowCrossOrigin(),
      },
    },
    expected: FetchTestResult.SUCCESS,
  });
}, "public to local: success.");

promise_test(async t => {
  let uuid = token();
  await fetchTest(t, {
    source: { server: Server.HTTPS_PUBLIC },
    target: {
      server: Server.HTTPS_PRIVATE,
      behavior: {
        preflight: PreflightBehavior.singlePreflight(uuid),
        response: ResponseBehavior.allowCrossOrigin(),
      },
    },
    expected: FetchTestResult.SUCCESS,
  });
  await fetchTest(t, {
    source: { server: Server.HTTPS_PUBLIC },
    target: {
      server: Server.HTTPS_PRIVATE,
      behavior: {
        preflight: PreflightBehavior.singlePreflight(uuid),
        response: ResponseBehavior.allowCrossOrigin(),
      },
    },
    expected: FetchTestResult.SUCCESS,
  });
}, "public to private: success.");