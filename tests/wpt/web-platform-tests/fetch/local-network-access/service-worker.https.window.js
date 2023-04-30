// META: script=/common/utils.js
// META: script=resources/support.sub.js
//
// Spec: https://wicg.github.io/private-network-access/#integration-fetch
//
// These tests check that initial `ServiceWorker` script fetches are exempt from
// Private Network Access checks because they are always same-origin and the
// origin is potentially trustworthy.
//
// See also: worker.https.window.js

// Results that may be expected in tests.
const TestResult = {
  SUCCESS: {
    register: { loaded: true },
    unregister: { unregistered: true },
  },
  FAILURE: {
    register: { error: "TypeError" },
    unregister: { unregistered: false, error: "no registration" },
  },
};

async function makeTest(t, { source, target, expected }) {
  const sourceUrl = resolveUrl("resources/service-worker-bridge.html",
                               sourceResolveOptions(source));

  const targetUrl = preflightUrl(target);
  targetUrl.searchParams.append("body", "undefined");
  targetUrl.searchParams.append("mime-type", "application/javascript");

  const scope = resolveUrl(`resources/${token()}`, {...target.server}).href;

  const iframe = await appendIframe(t, document, sourceUrl);

  {
    const reply = futureMessage();
    const message = {
      action: "register",
      url: targetUrl.href,
      options: { scope },
    };
    iframe.contentWindow.postMessage(message, "*");

    const { error, loaded } = await reply;

    assert_equals(error, expected.register.error, "register error");
    assert_equals(loaded, expected.register.loaded, "response loaded");
  }

  {
    const reply = futureMessage();
    iframe.contentWindow.postMessage({ action: "unregister", scope }, "*");

    const { error, unregistered } = await reply;
    assert_equals(error, expected.unregister.error, "unregister error");
    assert_equals(
        unregistered, expected.unregister.unregistered, "worker unregistered");
  }
}

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
    server: Server.HTTPS_PRIVATE,
    treatAsPublic: true,
  },
  target: { server: Server.HTTPS_PRIVATE },
  expected: TestResult.SUCCESS,
}), "treat-as-public to private: success.");

promise_test(t => makeTest(t, {
  source: { server: Server.HTTPS_PUBLIC },
  target: { server: Server.HTTPS_PUBLIC },
  expected: TestResult.SUCCESS,
}), "public to public: success.");
