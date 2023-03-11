// META: script=/common/utils.js
// META: script=resources/support.sub.js
//
// Spec: https://wicg.github.io/private-network-access/#integration-fetch
//
// These tests check that `ServiceWorker` script update fetches are subject to
// Private Network Access checks, just like regular `fetch()` calls. The client
// of the fetch, for PNA purposes, is taken to be the previous script.
//
// The tests is carried out by instantiating a service worker from a resource
// that carries the `Content-Security-Policy: treat-as-public-address` header,
// such that the registration is placed in the public IP address space. When
// the script is fetched for an update, the client is thus considered public,
// yet the same-origin fetch observes that the server's IP endpoint is not
// necessarily in the public IP address space.
//
// See also: worker.https.window.js

// Results that may be expected in tests.
const TestResult = {
  SUCCESS: { updated: true },
  FAILURE: { error: "TypeError" },
};

async function makeTest(t, { target, expected }) {
  // The bridge must be same-origin with the service worker script.
  const bridgeUrl = resolveUrl(
      "resources/service-worker-bridge.html",
      sourceResolveOptions({ server: target.server }));

  const scriptUrl = preflightUrl(target);
  scriptUrl.searchParams.append("treat-as-public-once", token());
  scriptUrl.searchParams.append("mime-type", "application/javascript");
  scriptUrl.searchParams.append("file", "service-worker.js");
  scriptUrl.searchParams.append("random-js-prefix", true);

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
    let { controlled, numControllerChanges } = await request({
      action: "wait",
      numControllerChanges: 1,
    });

    assert_equals(numControllerChanges, 1, "controller change");
    assert_true(controlled, "bridge script is controlled");

    const { error, updated } = await request({ action: "update" });

    assert_equals(error, expected.error, "update error");
    assert_equals(updated, expected.updated, "registration updated");

    // Stop here if we do not expect the update to succeed.
    if (!expected.updated) {
      return;
    }

    ({ controlled, numControllerChanges } = await request({
      action: "wait",
      numControllerChanges: 2,
    }));

    assert_equals(numControllerChanges, 2, "controller change");
    assert_true(controlled, "bridge script still controlled");
  } finally {
    const { error, unregistered } = await request({
      action: "unregister",
      scope: new URL("./", scriptUrl).href,
    });

    assert_equals(error, undefined, "unregister error");
    assert_true(unregistered, "unregistered");
  }
}

promise_test(t => makeTest(t, {
  target: { server: Server.HTTPS_LOCAL },
  expected: TestResult.FAILURE,
}), "update public to local: failed preflight.");

promise_test(t => makeTest(t, {
  target: {
    server: Server.HTTPS_LOCAL,
    behavior: { preflight: PreflightBehavior.serviceWorkerSuccess(token()) },
  },
  expected: TestResult.SUCCESS,
}), "update public to local: success.");

promise_test(t => makeTest(t, {
  target: { server: Server.HTTPS_PRIVATE },
  expected: TestResult.FAILURE,
}), "update public to private: failed preflight.");

promise_test(t => makeTest(t, {
  target: {
    server: Server.HTTPS_PRIVATE,
    behavior: { preflight: PreflightBehavior.serviceWorkerSuccess(token()) },
  },
  expected: TestResult.SUCCESS,
}), "update public to private: success.");

promise_test(t => makeTest(t, {
  target: { server: Server.HTTPS_PUBLIC },
  expected: TestResult.SUCCESS,
}), "update public to public: success.");
