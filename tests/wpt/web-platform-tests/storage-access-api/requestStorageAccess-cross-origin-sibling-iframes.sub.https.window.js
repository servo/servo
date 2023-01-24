// META: script=helpers.js
// META: script=/cookies/resources/cookie-helper.sub.js
// META: script=/resources/testdriver.js
// META: script=/resources/testdriver-vendor.js
'use strict';

(async function() {
  // This is on the www subdomain, so it's cross-origin from the current document.
  const wwwAlt = "https://{{hosts[alt][www]}}:{{ports[https][0]}}";

  promise_test(async (t) => {
    const responder_html = `${wwwAlt}/storage-access-api/resources/script-with-cookie-header.py?script=embedded_responder.js`;
    const [frame1, frame2] = await Promise.all([
      CreateFrame(responder_html),
      CreateFrame(responder_html),
    ]);

    t.add_cleanup(async () => {
      await test_driver.delete_all_cookies();
      await SetPermissionInFrame(frame1, [{ name: 'storage-access' }, 'prompt']);
    });

    await SetPermissionInFrame(frame1, [{ name: 'storage-access' }, 'granted']);
    await fetch(`${wwwAlt}/cookies/resources/set.py?cookie=monster;Secure;SameSite=None;Path=/`,
      { mode: "no-cors", credentials: "include" }).then((resp) => resp.text());

    await MaybeSetStorageAccess(wwwAlt + "/", "*", "blocked");
    t.add_cleanup(async () => {
      await MaybeSetStorageAccess(wwwAlt + "/", "*", "allowed");
    });

    assert_false(await FrameHasStorageAccess(frame1), "frame1 should not have storage access initially.");
    assert_false(await FrameHasStorageAccess(frame2), "frame2 should not have storage access initially.");

    assert_false(cookieStringHasCookie("cookie", "monster", await GetJSCookiesFromFrame(frame1)), "frame1 should not have access via document.cookie.");
    assert_false(cookieStringHasCookie("cookie", "monster", await GetJSCookiesFromFrame(frame2)), "frame2 should not have access via document.cookie.");

    assert_true(await RequestStorageAccessInFrame(frame1), "requestStorageAccess doesn't require a gesture since the permission has already been granted.");

    assert_true(await FrameHasStorageAccess(frame1), "frame1 should have storage access now.");
    assert_true(cookieStringHasCookie("cookie", "monster", await GetJSCookiesFromFrame(frame1)), "frame1 should now be able to access cookies via document.cookie.");

    assert_false(await FrameHasStorageAccess(frame2), "frame2 should still not have storage access.");
    assert_false(cookieStringHasCookie("cookie", "monster", await GetJSCookiesFromFrame(frame2)), "frame2 should should still be unable to access cookies via document.cookie");

    assert_true(await RequestStorageAccessInFrame(frame2), "frame2 should be able to get storage access without a gesture.");

    assert_true(await FrameHasStorageAccess(frame2), "frame2 should have storage access after it requested it.");
    assert_true(cookieStringHasCookie("cookie", "monster", await GetJSCookiesFromFrame(frame2)), "frame2 should be able to access cookies via document.cookie after getting storage access.");
  }, "Grants have per-frame scope");
})();
