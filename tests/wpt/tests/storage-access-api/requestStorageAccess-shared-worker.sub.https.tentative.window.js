// META: script=helpers.js
// META: script=/cookies/resources/cookie-helper.sub.js
// META: script=/resources/testdriver.js
// META: script=/resources/testdriver-vendor.js
'use strict';

// A SharedWorker created by a frame that has storage access must not inherit that
// access, matching dedicated workers (see requestStorageAccess-dedicated-worker).
(function() {
  const altRoot = "https://{{hosts[alt][]}}:{{ports[https][0]}}";
  const altRootWss = "wss://{{hosts[alt][]}}:{{ports[wss][0]}}";

  const responderPath = "/storage-access-api/resources/script-with-cookie-header.py?script=embedded_worker_responder.js";
  const echoCookiesPath = `/storage-access-api/resources/echo-cookie-header.py`;

  const altRootResponder = `${altRoot}${responderPath}`;
  const altRootEchoCookies = `${altRoot}${echoCookiesPath}`;

  async function SetUpResponderFrame(t, url) {
    const frame = await CreateFrame(url);

    await SetPermissionInFrame(frame, [{ name: 'storage-access' }, 'granted']);
    t.add_cleanup(async () => {
      await test_driver.delete_all_cookies();
      await SetPermissionInFrame(frame, [{ name: 'storage-access' }, 'prompt']);
      await MaybeSetStorageAccess("*", "*", "allowed");
    });

    return frame;
  }

  async function StartWorker(t, frame) {
    await PostMessageAndAwaitReply({command: "start_shared_worker", name: `${t.name}-${Date.now()}`}, frame.contentWindow);
  }

  function MessageWorker(frame, message) {
    return PostMessageAndAwaitReply({command: "message_shared_worker", message}, frame.contentWindow);
  }

  promise_test(async (t) => {
    await MaybeSetStorageAccess("*", "*", "blocked");
    await SetFirstPartyCookie(altRoot);

    const frame = await SetUpResponderFrame(t, altRootResponder);
    assert_true(await RequestStorageAccessInFrame(frame), "requestStorageAccess resolves without requiring a gesture.");
    assert_true(await FrameHasStorageAccess(frame), "frame has storage access after request.");
    assert_true(await HasUnpartitionedCookie(frame), "frame has access to cookies after request.");

    await StartWorker(t, frame);

    assert_false(cookieStringHasCookie("cookie", "unpartitioned",
          await MessageWorker(frame, {command: "fetch", url: altRootEchoCookies})),
        "Worker's fetch is uncredentialed.");
  }, "SharedWorkers don't inherit storage access for fetch");

  promise_test(async (t) => {
    await MaybeSetStorageAccess("*", "*", "blocked");
    await SetFirstPartyCookie(altRoot);

    const frame = await SetUpResponderFrame(t, altRootResponder);
    assert_false(await FrameHasStorageAccess(frame), "frame lacks storage access before request.");

    await StartWorker(t, frame);
    assert_false(cookieStringHasCookie("cookie", "unpartitioned",
          await MessageWorker(frame, {command: "fetch", url: altRootEchoCookies})),
        "Worker's first fetch is uncredentialed.");

    // Since the parent document obtains storage access *after* having created
    // the worker, this should have no effect on the worker.
    assert_true(await RequestStorageAccessInFrame(frame), "requestStorageAccess resolves without requiring a gesture.");
    assert_true(await FrameHasStorageAccess(frame), "frame has storage access after request.");
    assert_true(await HasUnpartitionedCookie(frame), "frame has access to cookies after request.");

    assert_false(cookieStringHasCookie("cookie", "unpartitioned",
          await MessageWorker(frame, {command: "fetch", url: altRootEchoCookies})),
        "Worker's second fetch is uncredentialed.");
  }, "SharedWorkers don't observe parent's storage access");

  promise_test(async (t) => {
    await MaybeSetStorageAccess("*", "*", "blocked");
    await SetFirstPartyCookie(altRoot);

    const frame = await SetUpResponderFrame(t, altRootResponder);
    assert_true(await RequestStorageAccessInFrame(frame), "requestStorageAccess resolves without requiring a gesture.");
    assert_true(await HasUnpartitionedCookie(frame), "frame has access to cookies after request.");
    assert_true(cookieStringHasCookie("cookie", "unpartitioned",
          await ReadCookiesFromWebSocketConnection(frame, altRootWss)),
        "Control: the frame's own WebSocket handshake includes the unpartitioned cookie.");

    await StartWorker(t, frame);

    assert_false(cookieStringHasCookie("cookie", "unpartitioned",
          await MessageWorker(frame, {command: "websocket", origin: altRootWss})),
        "Worker's WebSocket handshake omits the unpartitioned cookie.");
  }, "SharedWorker WebSocket doesn't inherit storage access");
}());
