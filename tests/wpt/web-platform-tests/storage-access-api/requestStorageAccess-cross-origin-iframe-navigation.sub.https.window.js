// META: script=helpers.js
// META: script=/cookies/resources/cookie-helper.sub.js
// META: script=/resources/testdriver.js
// META: script=/resources/testdriver-vendor.js
'use strict';

(async function() {
  // This is on the www subdomain, so it's cross-origin from the current document.
  const wwwHost = "https://{{domains[www]}}:{{ports[https][0]}}";

  // Set up storage access rules
  try {
    await test_driver.set_storage_access(wwwHost + "/", "*", "blocked");
  } catch (e) {
    // Ignore, can be unimplemented if the platform blocks cross-site cookies
    // by default. If this failed without default blocking we'll notice it later
    // in the test.
  }

  promise_test(async (t) => {
    const responder_html = `${wwwHost}/storage-access-api/resources/script-with-cookie-header.py?script=embedded_responder.js`;
    const frame = await CreateFrame(responder_html);

    t.add_cleanup(async () => {
      await test_driver.delete_all_cookies();
      await SetPermissionInFrame(frame, [{ name: 'storage-access' }, 'prompt']);
    });

    await SetPermissionInFrame(frame, [{ name: 'storage-access' }, 'granted']);
    await fetch(`${wwwHost}/cookies/resources/set.py?cookie=monster;Secure;SameSite=None;Path=/`,
      { mode: "no-cors", credentials: "include" });

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
