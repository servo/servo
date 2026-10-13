// META: title=Overwriting a cookie's HttpOnly attribute while its value stays the same

'use strict';

// https://httpwg.org/http-extensions/draft-ietf-httpbis-layered-cookies.html#store-a-cookie
// https://github.com/httpwg/http-extensions/issues/3501

async function setCookieViaHTTP(cookie) {
  const set = encodeURIComponent(JSON.stringify(cookie));
  const response = await fetch(`/cookies/resources/cookie.py?set=${set}`);
  assert_true(response.ok, 'Setting the cookie via HTTP succeeded');
}

// The raw Cookie request header, which unlike document.cookie also observes
// HttpOnly cookies.
async function getCookieHeader() {
  const response = await fetch('/cookiestore/resources/cookie_helper.py',
                               {credentials: 'include'});
  assert_true(response.ok, 'Reading the Cookie header succeeded');
  const text = await response.text();
  return decodeURIComponent(text.replace(/^cookie=/, ''));
}

function cookieValue(cookieString, name) {
  for (const pair of cookieString.split('; ')) {
    const index = pair.indexOf('=');
    if (index !== -1 && pair.substring(0, index) === name) {
      return pair.substring(index + 1);
    }
  }
  return null;
}

// Whether a cookie's http-only is true, observed without reading
// document.cookie: a cookie can only be overwritten through a non-HTTP API if
// its http-only is false, so if a write from script does not land the cookie's
// http-only is true.
async function isHttpOnly(name) {
  document.cookie = `${name}=overwritten; Path=/`;
  return cookieValue(await getCookieHeader(), name) !== 'overwritten';
}

function cookieTest(description, suffix, body) {
  const name = `httponly-overwrite-${suffix}`;
  promise_test(async t => {
    t.add_cleanup(() => setCookieViaHTTP(`${name}=; Path=/; Max-Age=0`));
    t.add_cleanup(
        () => setCookieViaHTTP(`${name}=; Path=/; Max-Age=0; HttpOnly`));
    await body(t, name);
  }, description);
}

// These two establish that isHttpOnly() reports what it claims to.
cookieTest('A cookie set without HttpOnly is not http-only', 'control1',
           async (t, name) => {
  await setCookieViaHTTP(`${name}=1; Path=/`);
  assert_false(await isHttpOnly(name));
});

cookieTest('A cookie set with HttpOnly is http-only', 'control2',
           async (t, name) => {
  await setCookieViaHTTP(`${name}=1; Path=/; HttpOnly`);
  assert_true(await isHttpOnly(name));
});

// A cookie's http-only has to be updated even when nothing else about the cookie
// changes. Failing to do so leaves the cookie writable by the non-HTTP APIs the
// server just asked to have it protected from.
cookieTest('Adding HttpOnly to a cookie with an unchanged value makes it http-only',
           'test1', async (t, name) => {
  await setCookieViaHTTP(`${name}=1; Path=/`);
  assert_false(await isHttpOnly(name), 'The cookie starts out not http-only');

  await setCookieViaHTTP(`${name}=1; Path=/; HttpOnly`);
  assert_equals(cookieValue(await getCookieHeader(), name), '1',
                'The cookie is still in the cookie store with its value');
  assert_true(await isHttpOnly(name), 'The cookie is now http-only');
});

cookieTest('Removing HttpOnly from a cookie with an unchanged value makes it not http-only',
           'test2', async (t, name) => {
  await setCookieViaHTTP(`${name}=1; Path=/; HttpOnly`);
  assert_true(await isHttpOnly(name), 'The cookie starts out http-only');

  await setCookieViaHTTP(`${name}=1; Path=/`);
  assert_equals(cookieValue(await getCookieHeader(), name), '1',
                'The cookie is still in the cookie store with its value');
  assert_false(await isHttpOnly(name), 'The cookie is no longer http-only');
});

// As above, but with both cookies in a single response, so that they also have
// an identical creation time.
cookieTest('Adding HttpOnly through a second Set-Cookie in the same response',
           'test3', async (t, name) => {
  await setCookieViaHTTP([`${name}=1; Path=/`, `${name}=1; Path=/; HttpOnly`]);
  assert_equals(cookieValue(await getCookieHeader(), name), '1',
                'The cookie is in the cookie store with its value');
  assert_true(await isHttpOnly(name), 'The cookie is http-only');
});

cookieTest('Removing HttpOnly through a second Set-Cookie in the same response',
           'test4', async (t, name) => {
  await setCookieViaHTTP([`${name}=1; Path=/; HttpOnly`, `${name}=1; Path=/`]);
  assert_equals(cookieValue(await getCookieHeader(), name), '1',
                'The cookie is in the cookie store with its value');
  assert_false(await isHttpOnly(name), 'The cookie is not http-only');
});

// An http-only cookie also has to stop being exposed to non-HTTP APIs. This is
// separate from the tests above so that a user agent that protects the cookie
// from being overwritten but keeps exposing its value fails only here.
cookieTest('Adding HttpOnly to a cookie with an unchanged value hides it from document.cookie',
           'test5', async (t, name) => {
  await setCookieViaHTTP(`${name}=1; Path=/`);
  assert_equals(cookieValue(document.cookie, name), '1',
                'The cookie is exposed to document.cookie');

  await setCookieViaHTTP(`${name}=1; Path=/; HttpOnly`);
  assert_equals(cookieValue(await getCookieHeader(), name), '1',
                'The cookie is still in the cookie store with its value');
  assert_equals(cookieValue(document.cookie, name), null,
                'The cookie is no longer exposed to document.cookie');
});

cookieTest('Removing HttpOnly from a cookie with an unchanged value exposes it to document.cookie',
           'test6', async (t, name) => {
  await setCookieViaHTTP(`${name}=1; Path=/; HttpOnly`);
  assert_equals(cookieValue(document.cookie, name), null,
                'The cookie is not exposed to document.cookie');

  await setCookieViaHTTP(`${name}=1; Path=/`);
  assert_equals(cookieValue(await getCookieHeader(), name), '1',
                'The cookie is still in the cookie store with its value');
  assert_equals(cookieValue(document.cookie, name), '1',
                'The cookie is now exposed to document.cookie');
});
