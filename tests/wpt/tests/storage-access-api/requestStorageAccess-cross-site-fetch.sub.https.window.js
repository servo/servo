// META: script=helpers.js
// META: script=/cookies/resources/cookie-helper.sub.js
// META: script=/resources/testdriver.js
// META: script=/resources/testdriver-vendor.js
'use strict';

const mainHost = "https://{{host}}:{{ports[https][0]}}";
const altRoot = "https://{{hosts[alt][]}}:{{ports[https][0]}}";
const responderPath = "/storage-access-api/resources/script-with-cookie-header.py?script=embedded_responder.js";

const altRootResponder = `${altRoot}${responderPath}`;
const domainCookieString = "cookie=unpartitioned;Secure;SameSite=None;Path=/;Domain={{hosts[alt][]}}";

async function SetUpResponderFrame(t, url) {
  const frame = await CreateFrame(url);

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
      await FetchSubresourceCookiesFromFrame(frame, altRoot));
  if (initiallyHasCookieAccess) {
    // Nothing to test here; third-party cookies are already accessible.
    return;
  }

  await SetPermissionInFrame(frame, [{ name: 'storage-access' }, 'granted']);

  assert_true(await RequestStorageAccessInFrame(frame), "requestStorageAccess resolves without requiring a gesture.");
  assert_true(await FrameHasStorageAccess(frame), "frame has storage access after request.");
  await SetDocumentCookieFromFrame(frame, domainCookieString);
  assert_true(await HasUnpartitionedCookie(frame), "frame has access to cookies after request.");

  // Redirect back to the iframe's origin, via a cross-site redirect. The
  // frame's origin is `{{hosts[alt][]}}`, so `{{host}}` is cross-site to it.
  const dest = `${altRoot}/storage-access-api/resources/echo-cookie-header.py`;
  const redirect = `${mainHost}/common/redirect.py?enable-cors=&location=${encodeURIComponent(dest)}`;
  assert_false(
    cookieStringHasCookie("cookie", "unpartitioned",
      await FetchFromFrame(frame, redirect)),
      "fetch is not credentialed after a cross-site redirect");
}, "Cross-site HTTP redirects from a frame with storage-access are not credentialed by default");
