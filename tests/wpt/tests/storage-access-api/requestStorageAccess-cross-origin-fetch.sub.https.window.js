// META: script=helpers.js
// META: script=/cookies/resources/cookie-helper.sub.js
// META: script=/resources/testdriver.js
// META: script=/resources/testdriver-vendor.js
'use strict';

(async function() {
  // These are cross-site from the current document.
  const altWww = "https://{{hosts[alt][www]}}:{{ports[https][0]}}";
  const altRoot = "https://{{hosts[alt][]}}:{{ports[https][0]}}";
  const responderPath = "/storage-access-api/resources/script-with-cookie-header.py?script=embedded_responder.js";

  const altRootResponder = `${altRoot}${responderPath}`;
  const domainCookieString = "cookie=unpartitioned;Secure;SameSite=None;Path=/;Domain={{hosts[alt][]}}";

  async function SetUpResponderFrame(t, url) {
    const frame = await CreateFrame(url);

    await SetPermissionInFrame(frame, [{ name: 'storage-access' }, 'granted']);
    t.add_cleanup(async () => {
      await test_driver.delete_all_cookies();
      await SetPermissionInFrame(frame, [{ name: 'storage-access' }, 'prompt']);
      await DeleteCookieInFrame(frame, "cookie", "Secure;SameSite=None;Path=/;Domain={{hosts[alt][]}}");
    });

    return frame;
  }

  promise_test(async (t) => {
    await SetFirstPartyCookie(altRoot, "initial-cookie=unpartitioned;Secure;SameSite=None;Path=/");
    const frame = await SetUpResponderFrame(t, altRootResponder);
    await SetDocumentCookieFromFrame(frame, domainCookieString);

    const initiallyHasCookieAccess =
      cookieStringHasCookie("cookie", "unpartitioned",
        await FetchSubresourceCookiesFromFrame(frame, altWww));
    if (initiallyHasCookieAccess) {
      // Nothing to test here; third-party cookies are already accessible.
      return;
    }

    assert_true(await RequestStorageAccessInFrame(frame), "requestStorageAccess resolves without requiring a gesture.");
    assert_true(await FrameHasStorageAccess(frame), "frame has storage access after request.");
    await SetDocumentCookieFromFrame(frame, domainCookieString);
    assert_true(await HasUnpartitionedCookie(frame), "frame has access to cookies after request.");

    // The frame's origin is hosts[alt][], so hosts[alt][www] is same-site but
    // cross-origin to it.
    assert_false(
        cookieStringHasCookie("cookie", "unpartitioned",
          await FetchSubresourceCookiesFromFrame(frame, altWww)),
        "same-site cross-origin fetch is not credentialed");
  }, "Cross-origin fetches from a frame with storage-access are not credentialed by default");

  promise_test(async (t) => {
    await SetFirstPartyCookie(altRoot, "initial-cookie=unpartitioned;Secure;SameSite=None;Path=/");
    const frame = await SetUpResponderFrame(t, altRootResponder);
    await SetDocumentCookieFromFrame(frame, domainCookieString);

    const initiallyHasCookieAccess =
      cookieStringHasCookie("cookie", "unpartitioned",
        await FetchSubresourceCookiesFromFrame(frame, altWww));
    if (initiallyHasCookieAccess) {
      // Nothing to test here; third-party cookies are already accessible.
      return;
    }

    assert_true(await RequestStorageAccessInFrame(frame), "requestStorageAccess resolves without requiring a gesture.");
    assert_true(await FrameHasStorageAccess(frame), "frame has storage access after request.");
    await SetDocumentCookieFromFrame(frame, domainCookieString);
    assert_true(await HasUnpartitionedCookie(frame), "frame has access to cookies after request.");

    // The frame's origin is hosts[alt][], so hosts[alt][www] is same-site but
    // cross-origin to it.
    const cross_origin_redirect = `${altRoot}/common/redirect.py?location=${altWww}/storage-access-api/resources/echo-cookie-header.py`;
    assert_false(
        cookieStringHasCookie("cookie", "unpartitioned",
          await FetchFromFrame(frame, cross_origin_redirect)),
        "fetch is not credentialed after a cross-origin redirect");
  }, "Cross-origin HTTP redirects from a frame with storage-access are not credentialed by default");

})();
