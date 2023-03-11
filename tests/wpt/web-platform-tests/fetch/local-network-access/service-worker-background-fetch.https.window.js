// META: script=/common/utils.js
// META: script=resources/support.sub.js
//
// Spec: https://wicg.github.io/private-network-access/#integration-fetch
// Spec: https://wicg.github.io/background-fetch/
//
// These tests check that background fetches from within `ServiceWorker` scripts
// are not subject to Private Network Access checks.

// Results that may be expected in tests.
const TestResult = {
  SUCCESS: { ok: true, body: "success", result: "success", failureReason: "" },
};

async function makeTest(t, { source, target, expected }) {
  const scriptUrl =
      resolveUrl("resources/service-worker.js", sourceResolveOptions(source));

  const bridgeUrl = new URL("service-worker-bridge.html", scriptUrl);

  const targetUrl = preflightUrl(target);

  const iframe = await appendIframe(t, document, bridgeUrl);

  const request = (message) => {
    const reply = futureMessage();
    iframe.contentWindow.postMessage(message, "*");
    return reply;
  };

  {
    const { error, loaded } = await request({
      action: "register",
      url: scriptUrl.href,
    });

    assert_equals(error, undefined, "register error");
    assert_true(loaded, "response loaded");
  }

  {
    const { error, state } = await request({
      action: "set-permission",
      name: "background-fetch",
      state: "granted",
    });

    assert_equals(error, undefined, "set permission error");
    assert_equals(state, "granted", "permission state");
  }

  {
    const { error, result, failureReason, ok, body } = await request({
      action: "background-fetch",
      url: targetUrl.href,
    });

    assert_equals(error, expected.error, "error");
    assert_equals(failureReason, expected.failureReason, "fetch failure reason");
    assert_equals(result, expected.result, "fetch result");
    assert_equals(ok, expected.ok, "response ok");
    assert_equals(body, expected.body, "response body");
  }
}

promise_test(t => makeTest(t, {
  source: { server: Server.HTTPS_LOCAL },
  target: { server: Server.HTTPS_LOCAL },
  expected: TestResult.SUCCESS,
}), "local to local: success.");

promise_test(t => makeTest(t, {
  source: { server: Server.HTTPS_PRIVATE },
  target: {
    server: Server.HTTPS_LOCAL,
    behavior: { response: ResponseBehavior.allowCrossOrigin() },
  },
  expected: TestResult.SUCCESS,
}), "private to local: success.");

promise_test(t => makeTest(t, {
  source: { server: Server.HTTPS_PRIVATE },
  target: { server: Server.HTTPS_PRIVATE },
  expected: TestResult.SUCCESS,
}), "private to private: success.");

promise_test(t => makeTest(t, {
  source: { server: Server.HTTPS_PUBLIC },
  target: {
    server: Server.HTTPS_LOCAL,
    behavior: { response: ResponseBehavior.allowCrossOrigin() },
  },
  expected: TestResult.SUCCESS,
}), "public to local: success.");

promise_test(t => makeTest(t, {
  source: { server: Server.HTTPS_PUBLIC },
  target: {
    server: Server.HTTPS_PRIVATE,
    behavior: { response: ResponseBehavior.allowCrossOrigin() },
  },
  expected: TestResult.SUCCESS,
}), "public to private: success.");

promise_test(t => makeTest(t, {
  source: { server: Server.HTTPS_PUBLIC },
  target: { server: Server.HTTPS_PUBLIC },
  expected: TestResult.SUCCESS,
}), "public to public: success.");

promise_test(t => makeTest(t, {
  source: {
    server: Server.HTTPS_LOCAL,
    treatAsPublic: true,
  },
  target: { server: Server.HTTPS_LOCAL },
  expected: TestResult.SUCCESS,
}), "treat-as-public to local: success.");

promise_test(t => makeTest(t, {
  source: {
    server: Server.HTTPS_LOCAL,
    treatAsPublic: true,
  },
  target: {
    server: Server.HTTPS_PRIVATE,
    behavior: { response: ResponseBehavior.allowCrossOrigin() },
  },
  expected: TestResult.SUCCESS,
}), "treat-as-public to private: success.");

promise_test(t => makeTest(t, {
  source: {
    server: Server.HTTPS_LOCAL,
    treatAsPublic: true,
  },
  target: {
    server: Server.HTTPS_PUBLIC,
    behavior: { response: ResponseBehavior.allowCrossOrigin() },
  },
  expected: TestResult.SUCCESS,
}), "treat-as-public to public: success.");
