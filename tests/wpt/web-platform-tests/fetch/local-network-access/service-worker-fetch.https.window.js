// META: script=/common/utils.js
// META: script=resources/support.sub.js
//
// Spec: https://wicg.github.io/private-network-access/#integration-fetch
//
// These tests check that fetches from within `ServiceWorker` scripts are
// subject to Private Network Access checks, just like fetches from within
// documents.

// Results that may be expected in tests.
const TestResult = {
  SUCCESS: { ok: true, body: "success" },
  FAILURE: { error: "TypeError" },
};

async function makeTest(t, { source, target, expected }) {
  const bridgeUrl = resolveUrl(
      "resources/service-worker-bridge.html",
      sourceResolveOptions({ server: source.server }));

  const scriptUrl =
      resolveUrl("resources/service-worker.js", sourceResolveOptions(source));

  const realTargetUrl = preflightUrl(target);

  // Fetch a URL within the service worker's scope, but tell it which URL to
  // really fetch.
  const targetUrl = new URL("service-worker-proxy", scriptUrl);
  targetUrl.searchParams.append("proxied-url", realTargetUrl.href);

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

  try {
    const { controlled, numControllerChanges } = await request({
      action: "wait",
      numControllerChanges: 1,
    });

    assert_equals(numControllerChanges, 1, "controller change");
    assert_true(controlled, "bridge script is controlled");

    const { error, ok, body } = await request({
      action: "fetch",
      url: targetUrl.href,
    });

    assert_equals(error, expected.error, "fetch error");
    assert_equals(ok, expected.ok, "response ok");
    assert_equals(body, expected.body, "response body");
  } finally {
    // Always unregister the service worker.
    const { error, unregistered } = await request({
      action: "unregister",
      scope: new URL("./", scriptUrl).href,
    });

    assert_equals(error, undefined, "unregister error");
    assert_true(unregistered, "unregistered");
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
  expected: TestResult.FAILURE,
}), "private to local: failed preflight.");

promise_test(t => makeTest(t, {
  source: { server: Server.HTTPS_PRIVATE },
  target: {
    server: Server.HTTPS_LOCAL,
    behavior: {
      preflight: PreflightBehavior.success(token()),
      response: ResponseBehavior.allowCrossOrigin(),
    },
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
  expected: TestResult.FAILURE,
}), "public to local: failed preflight.");

promise_test(t => makeTest(t, {
  source: { server: Server.HTTPS_PUBLIC },
  target: {
    server: Server.HTTPS_LOCAL,
    behavior: {
      preflight: PreflightBehavior.success(token()),
      response: ResponseBehavior.allowCrossOrigin(),
    },
  },
  expected: TestResult.SUCCESS,
}), "public to local: success.");

promise_test(t => makeTest(t, {
  source: { server: Server.HTTPS_PUBLIC },
  target: {
    server: Server.HTTPS_PRIVATE,
    behavior: { response: ResponseBehavior.allowCrossOrigin() },
  },
  expected: TestResult.FAILURE,
}), "public to private: failed preflight.");

promise_test(t => makeTest(t, {
  source: { server: Server.HTTPS_PUBLIC },
  target: {
    server: Server.HTTPS_PRIVATE,
    behavior: {
      preflight: PreflightBehavior.success(token()),
      response: ResponseBehavior.allowCrossOrigin(),
    },
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
  expected: TestResult.FAILURE,
}), "treat-as-public to local: failed preflight.");

promise_test(t => makeTest(t, {
  source: {
    server: Server.HTTPS_LOCAL,
    treatAsPublic: true,
  },
  target: {
    server: Server.HTTPS_LOCAL,
    behavior: { preflight: PreflightBehavior.success(token()) },
  },
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
  expected: TestResult.FAILURE,
}), "treat-as-public to private: failed preflight.");

promise_test(t => makeTest(t, {
  source: {
    server: Server.HTTPS_LOCAL,
    treatAsPublic: true,
  },
  target: {
    server: Server.HTTPS_PRIVATE,
    behavior: {
      preflight: PreflightBehavior.success(token()),
      response: ResponseBehavior.allowCrossOrigin(),
    },
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
