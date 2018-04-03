'use strict';

// Set the secure example.org-domain cookie __Secure-COOKIENAME with
// value cookie-value on path /cgi-bin/ and 24 hour duration; domain
// and path will be rewritten below.
//
// This uses a Date object for expiration.
async function setOneDaySecureCookieWithDate() {
  // one day ahead, ignoring a possible leap-second
  let inTwentyFourHours = new Date(Date.now() + 24 * 60 * 60 * 1000);
  await cookieStore.set('__Secure-COOKIENAME', 'cookie-value', {
    path: kPath,
    expires: inTwentyFourHours,
    secure: true,
    domain: location.hostname
  });
}

// Set the secured example.org-domain cookie __Secure-COOKIENAME with
// value cookie-value on path /cgi-bin/ and expiration in June of next
// year; domain and path will be rewritten below.
//
// This uses an HTTP-style date string for expiration.
async function setSecureCookieWithHttpLikeExpirationString() {
  const year = (new Date()).getUTCFullYear() + 1;
  const date = new Date('07 Jun ' + year + ' 07:07:07 UTC');
  const day = ('Sun Mon Tue Wed Thu Fri Sat'.split(' '))[date.getUTCDay()];
  await cookieStore.set('__Secure-COOKIENAME', 'cookie-value', {
    path: kPath,
    expires: day + ', 07 Jun ' + year + ' 07:07:07 GMT',
    secure: true,
    domain: location.hostname
  });
}

// Set the unsecured example.org-domain cookie LEGACYCOOKIENAME with
// value cookie-value on path /cgi-bin/ and 24 hour duration; domain
// and path will be rewritten below.
//
// This uses milliseconds since the start of the Unix epoch for
// expiration.
async function setOneDayUnsecuredCookieWithMillisecondsSinceEpoch() {
  // one day ahead, ignoring a possible leap-second
  let inTwentyFourHours = Date.now() + 24 * 60 * 60 * 1000;
  await cookieStore.set('LEGACYCOOKIENAME', 'cookie-value', {
    path: kPath,
    expires: inTwentyFourHours,
    secure: false,
    domain: location.hostname
  });
}

// Delete the cookie written by
// setOneDayUnsecuredCookieWithMillisecondsSinceEpoch.
async function deleteUnsecuredCookieWithDomainAndPath() {
  await cookieStore.delete('LEGACYCOOKIENAME', {
    path: kPath,
    secure: false,
    domain: location.hostname
  });
}

cookie_test(async testCase => {
  await promise_rejects_when_unsecured(
    testCase,
    new TypeError(),
    setOneDaySecureCookieWithDate(),
    'Secure cookies only writable from secure contexts');

  const eventPromise = observeNextCookieChangeEvent();

  await setOneDayUnsecuredCookieWithMillisecondsSinceEpoch();
  assert_equals(
      await getCookieString('LEGACYCOOKIENAME'),
      'LEGACYCOOKIENAME=cookie-value',
      'Ensure unsecured cookie we set is visible');

  await verifyCookieChangeEvent(
    eventPromise,
    {changed: [{name: 'LEGACYCOOKIENAME', value: 'cookie-value'}]},
    'Ensure unsecured cookie we set is visible to observer');

  await deleteUnsecuredCookieWithDomainAndPath();
  await promise_rejects_when_unsecured(
      testCase,
      new TypeError(),
      setSecureCookieWithHttpLikeExpirationString(),
      'Secure cookies only writable from secure contexts');
}, 'expiration');
