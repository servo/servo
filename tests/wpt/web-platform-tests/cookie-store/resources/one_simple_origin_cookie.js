'use strict';

// Helper to verify first-of-name get using async/await.
//
// Returns the first script-visible value of the __Host-COOKIENAME cookie or
// undefined if no matching cookies are script-visible.
async function getOneSimpleOriginCookie() {
  let cookie = await cookieStore.get('__Host-COOKIENAME');
  if (!cookie) return undefined;
  return cookie.value;
}

// Returns the number of script-visible cookies whose names start with
// __Host-COOKIEN
async function countMatchingSimpleOriginCookies() {
  let cookieList = await cookieStore.getAll({
    name: '__Host-COOKIEN',
    matchType: 'startsWith'
  });
  return cookieList.length;
}

// Set the secure implicit-domain cookie __Host-COOKIENAME with value
// cookie-value on path / and session duration.
async function setOneSimpleOriginSessionCookie() {
  await cookieStore.set('__Host-COOKIENAME', 'cookie-value');
};

cookie_test(async testCase => {
  await promise_rejects_when_unsecured(
    testCase,
    new TypeError(),
    setOneSimpleOriginSessionCookie(),
    '__Host- prefix only writable from secure contexts');
  if (!kIsUnsecured) {
    assert_equals(
      await getOneSimpleOriginCookie(),
      'cookie-value',
      '__Host-COOKIENAME cookie should be found in a secure context');
  } else {
    assert_equals(
      await getOneSimpleOriginCookie(),
      undefined,
      '__Host-COOKIENAME cookie should not be found in an unsecured context');
  }
  if (kIsUnsecured) {
    assert_equals(
      await countMatchingSimpleOriginCookies(),
      0,
      'No __Host-COOKIEN* cookies should be found in an unsecured context');
  } else {
    assert_equals(
      await countMatchingSimpleOriginCookies(),
      1,
      'One __Host-COOKIEN* cookie should be found in a secure context');
  }
}, 'One simple origin cookie');
