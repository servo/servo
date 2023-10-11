// META: script=helpers.js
// META: script=/cookies/resources/cookie-helper.sub.js
// META: script=/resources/testdriver.js
// META: script=/resources/testdriver-vendor.js
'use strict';

(async function() {
  // This is cross-domain from the current document.
  const altWww = "https://{{hosts[alt][www]}}:{{ports[https][0]}}";
  const altRoot = "https://{{hosts[alt][]}}:{{ports[https][0]}}";
  const responderPath = "/storage-access-api/resources/script-with-cookie-header.py?script=embedded_responder.js";

  const altWwwResponder = `${altWww}${responderPath}`;
  const altRootResponder = `${altRoot}${responderPath}`;

  async function SetUpResponderFrame(t, url) {
    const frame = await CreateFrame(url);

    await SetPermissionInFrame(frame, [{ name: 'storage-access' }, 'granted']);
    t.add_cleanup(async () => {
      await test_driver.delete_all_cookies();
      await SetPermissionInFrame(frame, [{ name: 'storage-access' }, 'prompt']);
      await MaybeSetStorageAccess("*", "*", "allowed");
    });

    assert_false(await FrameHasStorageAccess(frame), "frame initially does not have storage access.");
    assert_false(await HasUnpartitionedCookie(frame), "frame initially does not have access to cookies.");

    assert_true(await RequestStorageAccessInFrame(frame), "requestStorageAccess resolves without requiring a gesture.");

    assert_true(await FrameHasStorageAccess(frame), "frame has storage access after request.");
    assert_true(await HasUnpartitionedCookie(frame), "frame has access to cookies after request.");

    return frame;
  }

  promise_test(async (t) => {
    await MaybeSetStorageAccess("*", "*", "blocked");
    await SetFirstPartyCookieAndUnsetStorageAccessPermission(altWww);

    const frame = await SetUpResponderFrame(t, altWwwResponder);

    await FrameInitiatedReload(frame);

    assert_true(await FrameHasStorageAccess(frame), "frame has storage access after refresh.");
    assert_true(await HasUnpartitionedCookie(frame), "frame has access to cookies after refresh.");
  }, "Self-initiated reloads preserve storage access");

  promise_test(async (t) => {
    await MaybeSetStorageAccess("*", "*", "blocked");
    await SetFirstPartyCookieAndUnsetStorageAccessPermission(altWww);

    const frame = await SetUpResponderFrame(t, altWwwResponder);

    await FrameInitiatedNavigation(frame, altWwwResponder);

    assert_true(await FrameHasStorageAccess(frame), "frame has storage access after refresh.");
    assert_true(await HasUnpartitionedCookie(frame), "frame has access to cookies after refresh.");
  }, "Self-initiated same-origin navigations preserve storage access");

  promise_test(async (t) => {
    await MaybeSetStorageAccess("*", "*", "blocked");
    await SetFirstPartyCookieAndUnsetStorageAccessPermission(altWww);

    const frame = await SetUpResponderFrame(t, altWwwResponder);

    await new Promise((resolve) => {
      frame.addEventListener("load", () => resolve());
      frame.src = altWwwResponder;
    });

    assert_false(await FrameHasStorageAccess(frame), "frame does not have storage access after refresh.");
    assert_false(await HasUnpartitionedCookie(frame), "frame has access to cookies after refresh.");
  }, "Non-self-initiated same-origin navigations do not preserve storage access");

  promise_test(async (t) => {
    await MaybeSetStorageAccess("*", "*", "blocked");
    await SetFirstPartyCookieAndUnsetStorageAccessPermission(altWww);

    const frame = await SetUpResponderFrame(t, altWwwResponder);

    await FrameInitiatedNavigation(frame, altRootResponder);

    assert_false(await FrameHasStorageAccess(frame), "frame does not have storage access after refresh.");
    assert_false(await HasUnpartitionedCookie(frame), "frame has access to cookies after refresh.");
  }, "Self-initiated cross-origin navigations do not preserve storage access");
})();
