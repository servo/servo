function testHttpCookies({desc, origin, cookieNames, expectsCookie}) {
  promise_test(async () => {
    await assertHttpOriginCanAccessCookies({ origin, cookieNames, expectsCookie });
  }, getCookieTestName(expectsCookie, desc, "HTTP"));
}

async function assertHttpOriginCanAccessCookies({
  origin,
  cookieNames,
  expectsCookie,
}) {
  const resp = await credFetch(`${origin}/cookies/resources/list.py`);
  const cookies = await resp.json();
  for (const cookieName of cookieNames) {
    assert_equals(
      cookies.hasOwnProperty(cookieName), expectsCookie,
      getCookieAssertDesc(expectsCookie, cookieName));
  }
}

async function assertThirdPartyHttpCookies({ desc, origin, cookieNames, expectsCookie }) {
  // Test that these cookies are not available on cross-site subresource requests to the
  // origin that set them.
  testHttpCookies({
    desc,
    origin,
    cookieNames,
    expectsCookie,
  });

  promise_test(async () => {
    const thirdPartyHttpCookie = "3P_http"
    await credFetch(
      `${origin}/cookies/resources/set.py?${thirdPartyHttpCookie}=foobar;` +
      "Secure;Path=/;SameSite=None");
    await assertHttpOriginCanAccessCookies({
      origin,
      cookieNames: [thirdPartyHttpCookie],
      expectsCookie,
    });
  }, desc + ": Cross site window setting HTTP cookies");
}

function testDomCookies({desc, cookieNames, expectsCookie}) {
  test(() => {
    assertDomCanAccessCookie(cookieNames, expectsCookie);
  }, getCookieTestName(expectsCookie, desc, "DOM"));
}

function assertDomCanAccessCookie(cookieNames, expectsCookie) {
  for (const cookieName of cookieNames) {
    assert_equals(
      document.cookie.includes(cookieName + "="), expectsCookie,
      getCookieAssertDesc(expectsCookie, cookieName));
  }
}

function testCookieStoreCookies({desc, cookieNames, expectsCookie}) {
  if (!window.cookieStore) return;
  promise_test(async () => {
    await assertCookieStoreCanAccessCookies(cookieNames, expectsCookie);
  }, getCookieTestName(expectsCookie, desc, "CookieStore"));
}

async function assertCookieStoreCanAccessCookies(cookieNames, expectsCookie) {
  const cookies = await cookieStore.getAll({sameSite: 'none'});
  for (const cookieName of cookieNames) {
    assert_equals(
      !!cookies.find(c => c.name === cookieName), expectsCookie,
      getCookieAssertDesc(expectsCookie, cookieName));
  }
}

function getCookieTestName(expectsCookie, desc, cookieType) {
  if (expectsCookie) {
      return `${desc}: Cookies are accessible via ${cookieType}`;
  }
  return `${desc}: Cookies are not accessible via ${cookieType}`;
}

function getCookieAssertDesc(expectsCookie, cookieName) {
  if (expectsCookie) {
    return `Expected cookie ${cookieName} to be available`;
  }
  return `Expected cookie ${cookieName} to not be available`;
}
