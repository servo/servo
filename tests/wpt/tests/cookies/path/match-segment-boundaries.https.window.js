// META: title=Cookie Path attribute matching only at path segment boundaries

'use strict';

// A cookie is only sent when its path is the request URL's path, or a prefix of
// it that ends at a path segment boundary. In particular a cookie path that is
// longer than the request URL's path never matches, even when the request URL's
// path is a prefix of it.
//
// https://httpwg.org/http-extensions/draft-ietf-httpbis-layered-cookies.html#store-a-cookie

const DIR = '/cookies/path/resources';
const TARGET = `${DIR}/echo.py`;
const NAME_PREFIX = 'segment-boundaries-';

async function setCookieViaHTTP(cookie) {
  const set = encodeURIComponent(JSON.stringify([cookie]));
  const response = await fetch(`/cookies/resources/cookie.py?set=${set}`);
  assert_true(response.ok, 'Setting the cookie via HTTP succeeded');
}

// The Cookie header the server receives for a request to TARGET.
async function cookieHeaderAtTarget() {
  const response = await fetch(TARGET, {credentials: 'include'});
  assert_true(response.ok, 'Reading the Cookie header succeeded');
  return (await response.text()).trim();
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

// Sets the cookie under test, then expires it again through the same Path, which
// is what removes it.
function pathTest({name, suffix, path, sent}) {
  const cookieName = NAME_PREFIX + suffix;
  promise_test(async t => {
    t.add_cleanup(
        () => setCookieViaHTTP(`${cookieName}=; Path=${path}; Max-Age=0`));

    await setCookieViaHTTP(`${cookieName}=1; Path=${path}`);
    assert_equals(cookieValue(await cookieHeaderAtTarget(), cookieName),
                  sent ? '1' : null);
  }, name);
}

pathTest({
  name: 'A Path equal to the request path matches',
  suffix: 'exact',
  path: TARGET,
  sent: true,
});

pathTest({
  name: 'A Path that is a prefix ending at a segment boundary matches',
  suffix: 'prefix',
  path: DIR,
  sent: true,
});

pathTest({
  name: 'A Path that is a prefix ending in a slash matches',
  suffix: 'trailingslash',
  path: `${DIR}/`,
  sent: true,
});

// The request path continues with ".py" rather than a "/", so the cookie path is
// not a prefix ending at a segment boundary.
pathTest({
  name: 'A Path that is a prefix not ending at a segment boundary does not match',
  suffix: 'midsegment',
  path: `${DIR}/echo`,
  sent: false,
});

// The request path is a prefix of the cookie path rather than the other way
// around. A cookie path longer than the request path can never match.
pathTest({
  name: 'A Path that is the request path followed by a slash does not match',
  suffix: 'longerslash',
  path: `${TARGET}/`,
  sent: false,
});

pathTest({
  name: 'A Path that is the request path followed by a segment does not match',
  suffix: 'longersegment',
  path: `${TARGET}/sub`,
  sent: false,
});
