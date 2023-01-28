// META: script=helpers.js
// META: script=/cookies/resources/cookie-helper.sub.js
// META: script=/resources/testdriver.js
// META: script=/resources/testdriver-vendor.js
'use strict';

(async function() {
  // This is cross-domain from the current document.
  const wwwAlt = "https://{{hosts[alt][www]}}:{{ports[https][0]}}";

  promise_test(async (t) => {
    const responder_html = `${wwwAlt}/storage-access-api/resources/script-with-cookie-header.py?script=embedded_responder.js`;
    const frame = await CreateFrame(responder_html);

    t.add_cleanup(async () => {
      await test_driver.delete_all_cookies();
      await SetPermissionInFrame(frame, [{ name: 'storage-access' }, 'prompt']);
    });

    await SetPermissionInFrame(frame, [{ name: 'storage-access' }, 'granted']);
    await fetch(`${wwwAlt}/cookies/resources/set.py?cookie=monster;Secure;SameSite=None;Path=/`,
      { mode: "no-cors", credentials: "include" }).then((resp) => resp.text());

    await MaybeSetStorageAccess(wwwAlt + "/", "*", "blocked");
    t.add_cleanup(async () => {
      await MaybeSetStorageAccess(wwwAlt + "/", "*", "allowed");
    });

    assert_false(await FrameHasStorageAccess(frame), "frame initially does not have storage access.");
    assert_false(cookieStringHasCookie("cookie", "monster", await GetJSCookiesFromFrame(frame)), "frame cannot access cookies via JS.");
    assert_false(cookieStringHasCookie("cookie", "monster", await GetHTTPCookiesFromFrame(frame)), "frame's fetch was done without credentials.");

    assert_true(await RequestStorageAccessInFrame(frame), "requestStorageAccess resolves without requiring a gesture.");

    assert_true(await FrameHasStorageAccess(frame), "frame has storage access after request.");
    assert_true(cookieStringHasCookie("cookie", "monster", await GetJSCookiesFromFrame(frame)), "frame has cookie access via JS after request.");

    await FrameInitiatedReload(frame);

    assert_true(await FrameHasStorageAccess(frame), "frame has storage access after refresh.");
    assert_true(cookieStringHasCookie("cookie", "monster", await GetJSCookiesFromFrame(frame)), "frame can access cookies via JS after refresh.");
    assert_true(cookieStringHasCookie("cookie", "monster", await GetHTTPCookiesFromFrame(frame)), "frame's fetch was credentialed.");
  }, "Self-initiated same-origin navigations preserve storage access");
})();
