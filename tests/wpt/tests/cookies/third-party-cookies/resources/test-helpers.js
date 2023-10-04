function testHttpCookies({desc, origin, cookieNames, expectsCookie}) {
  promise_test(async () => {
    await assertOriginCanAccessCookies({origin, cookieNames, expectsCookie});
  }, getCookieTestName(expectsCookie, desc, "HTTP"));
}

async function assertOriginCanAccessCookies({
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
