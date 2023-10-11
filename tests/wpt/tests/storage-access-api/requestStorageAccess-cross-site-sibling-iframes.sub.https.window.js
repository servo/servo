// META: script=helpers.js
// META: script=/cookies/resources/cookie-helper.sub.js
// META: script=/resources/testdriver.js
// META: script=/resources/testdriver-vendor.js
'use strict';

(async function() {
  // This is on the www subdomain, so it's cross-origin from the current document.
  const www = "https://{{domains[www]}}:{{ports[https][0]}}";
  // This is on the alt host, so it's cross-site from the current document.
  const wwwAlt = "https://{{hosts[alt][]}}:{{ports[https][0]}}";
  const url_suffix = "/storage-access-api/resources/script-with-cookie-header.py?script=embedded_responder.js";

  promise_test(async (t) => {
    await MaybeSetStorageAccess("*", "*", "blocked");
    await SetFirstPartyCookieAndUnsetStorageAccessPermission(wwwAlt);
    const responder_html = `${wwwAlt}${url_suffix}`;
    const [frame1, frame2] = await Promise.all([
      CreateFrame(responder_html),
      CreateFrame(responder_html),
    ]);

    t.add_cleanup(async () => {
      await test_driver.delete_all_cookies();
      await SetPermissionInFrame(frame1, [{ name: 'storage-access' }, 'prompt']);
      await MaybeSetStorageAccess("*", "*", "allowed");
    });

    await SetPermissionInFrame(frame1, [{ name: 'storage-access' }, 'granted']);

    assert_false(await FrameHasStorageAccess(frame1), "frame1 should not have storage access initially.");
    assert_false(await FrameHasStorageAccess(frame2), "frame2 should not have storage access initially.");

    assert_false(await HasUnpartitionedCookie(frame1), "frame1 should not have cookie access.");
    assert_false(await HasUnpartitionedCookie(frame2), "frame2 should not have cookie access.");

    assert_true(await RequestStorageAccessInFrame(frame1), "requestStorageAccess doesn't require a gesture since the permission has already been granted.");

    assert_true(await FrameHasStorageAccess(frame1), "frame1 should have storage access now.");
    assert_true(await HasUnpartitionedCookie(frame1), "frame1 should now have cookie access.");

    assert_false(await FrameHasStorageAccess(frame2), "frame2 should still not have storage access.");
    assert_false(await HasUnpartitionedCookie(frame2), "frame2 should still have cookie access.");

    assert_true(await RequestStorageAccessInFrame(frame2), "frame2 should be able to get storage access without a gesture.");

    assert_true(await FrameHasStorageAccess(frame2), "frame2 should have storage access after it requested it.");
    assert_true(await HasUnpartitionedCookie(frame2), "frame2 should have cookie access after getting storage access.");
  }, "Grants have per-frame scope");

  promise_test(async (t) => {
    await MaybeSetStorageAccess("*", "*", "blocked");
    const [crossOriginFrame, crossSiteFrame] = await Promise.all([
      CreateFrame(`${www}${url_suffix}`),
      CreateFrame(`${wwwAlt}${url_suffix}`),
    ]);

    t.add_cleanup(async () => {
      await test_driver.delete_all_cookies();
      await SetPermissionInFrame(crossOriginFrame, [{ name: 'storage-access' }, 'prompt']);
      await SetPermissionInFrame(crossSiteFrame, [{ name: 'storage-access' }, 'prompt']);
      await MaybeSetStorageAccess("*", "*", "allowed");
    });

    await SetPermissionInFrame(crossOriginFrame, [{ name: 'storage-access' }, 'granted']);
    await SetPermissionInFrame(crossSiteFrame, [{ name: 'storage-access' }, 'granted']);

    assert_true(await RequestStorageAccessInFrame(crossOriginFrame), "crossOriginFrame should be able to get storage access without a gesture.");
    assert_true(await RequestStorageAccessInFrame(crossSiteFrame), "crossSiteFrame should be able to get storage access without a gesture.");

    await SetDocumentCookieFromFrame(crossOriginFrame, `cookie=monster;Secure;SameSite=None;Path=/`);
    await SetDocumentCookieFromFrame(crossSiteFrame, `foo=bar;Secure;SameSite=None;Path=/`);

    assert_true(cookieStringHasCookie("cookie", "monster", await FetchSubresourceCookiesFromFrame(crossOriginFrame, www)),"crossOriginFrame making same-origin subresource request can access cookies.");
    assert_true(cookieStringHasCookie("foo", "bar", await FetchSubresourceCookiesFromFrame(crossSiteFrame, wwwAlt)),"crossSiteFrame making same-origin subresource request can access cookies.");

    assert_false(cookieStringHasCookie("foo", "bar",  await FetchSubresourceCookiesFromFrame(crossOriginFrame, wwwAlt)), "crossOriginFrame making cross-site subresource request to sibling iframe's host should not include cookies.");
    assert_false(cookieStringHasCookie("cookie", "monster", await FetchSubresourceCookiesFromFrame(crossSiteFrame, www)),"crossSiteFrame making cross-site subresource request to sibling iframe's host should not include cookies.");

  }, "Cross-site sibling iframes should not be able to take advantage of the existing permission grant requested by others.");

})();
