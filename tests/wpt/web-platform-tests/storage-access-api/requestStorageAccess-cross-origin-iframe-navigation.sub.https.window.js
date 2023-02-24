// META: script=helpers.js
// META: script=/cookies/resources/cookie-helper.sub.js
// META: script=/resources/testdriver.js
// META: script=/resources/testdriver-vendor.js
'use strict';

(async function() {
  // This is cross-domain from the current document.
  const wwwAlt = "https://{{hosts[alt][www]}}:{{ports[https][0]}}";

  promise_test(async (t) => {
    await MaybeSetStorageAccess(wwwAlt + "/", "*", "blocked");

    const responder_html = `${wwwAlt}/storage-access-api/resources/script-with-cookie-header.py?script=embedded_responder.js`;
    const frame = await CreateFrame(responder_html);

    await SetPermissionInFrame(frame, [{ name: 'storage-access' }, 'granted']);
    t.add_cleanup(async () => {
      await test_driver.delete_all_cookies();
      await SetPermissionInFrame(frame, [{ name: 'storage-access' }, 'prompt']);
      await MaybeSetStorageAccess(wwwAlt + "/", "*", "allowed");
    });

    assert_false(await FrameHasStorageAccess(frame), "frame initially does not have storage access.");
    assert_false(await CanFrameWriteCookies(frame), "frame initially cannot write cookies via document.cookie.");
    assert_false(cookieStringHasCookie("cookie", "monster", await GetHTTPCookiesFromFrame(frame)), "frame's fetch was done without credentials.");

    assert_true(await RequestStorageAccessInFrame(frame), "requestStorageAccess resolves without requiring a gesture.");

    assert_true(await FrameHasStorageAccess(frame), "frame has storage access after request.");
    assert_true(await CanFrameWriteCookies(frame), "frame can write cookies via JS after request.");

    await FrameInitiatedReload(frame);

    assert_true(await FrameHasStorageAccess(frame), "frame has storage access after refresh.");
    assert_true(await CanFrameWriteCookies(frame), "frame can write cookies via JS after refresh.");
  }, "Self-initiated same-origin navigations preserve storage access");
})();
