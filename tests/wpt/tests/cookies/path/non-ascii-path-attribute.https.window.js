// META: title=A non-ASCII byte in the winning Path attribute makes the cookie fail to parse

'use strict';

// A cookie's path is a URL path, whose segments are ASCII strings, so a Path
// attribute holding a non-ASCII byte cannot be represented and the cookie fails
// to parse. The check applies to the Path attribute that wins, not to every Path
// attribute seen, matching how the rest of a cookie is validated once parsing
// has settled on its final values.
//
// https://github.com/whatwg/url/issues/814

const DIR = '/cookies/path/resources';
const TARGET = `${DIR}/echo.py`;
// The default path for a cookie set through /cookies/resources/cookie.py.
const DEFAULT_PATH_TARGET = '/cookies/resources/list.py';
const NAME_PREFIX = 'non-ascii-path-';

// Sets a Set-Cookie header, replacing "ZZ" in `cookie` with the raw bytes given
// as percent-escapes. wptserve percent-decodes the query into bytes and
// cookie.py passes those bytes to the header unchanged, which is how a raw
// non-ASCII byte reaches the Path attribute.
async function setCookieViaHTTP(cookie, rawEscape) {
  let query = encodeURIComponent(JSON.stringify([cookie]));
  if (rawEscape !== undefined) {
    query = query.replace('ZZ', rawEscape);
  }
  const response = await fetch(`/cookies/resources/cookie.py?set=${query}`);
  assert_true(response.ok, 'Setting the cookie via HTTP succeeded');
}

// The value of `name` in the Cookie header the server receives for a request to
// TARGET, or null if it is not there.
async function cookieValueAtTarget(name) {
  const response = await fetch(TARGET, {credentials: 'include'});
  assert_true(response.ok, 'Reading the Cookie header succeeded');
  for (const pair of (await response.text()).trim().split('; ')) {
    const index = pair.indexOf('=');
    if (index !== -1 && pair.substring(0, index) === name) {
      return pair.substring(index + 1);
    }
  }
  return null;
}

// The same for a request to the default path, which answers with JSON.
async function cookieValueAtDefaultPath(name) {
  const response = await fetch(DEFAULT_PATH_TARGET, {credentials: 'include'});
  assert_true(response.ok, 'Reading the cookies succeeded');
  const cookies = await response.json();
  return name in cookies ? cookies[name] : null;
}

// Cookies are expired through every Path they could have been stored under,
// since only a matching Path removes them.
function cookieTest(description, suffix, paths, body) {
  const name = NAME_PREFIX + suffix;
  promise_test(async t => {
    for (const path of paths) {
      t.add_cleanup(
          () => setCookieViaHTTP(`${name}=; Path=${path}; Max-Age=0`));
    }
    await body(t, name);
  }, description);
}

// Control: a later Path attribute overrides an earlier one, which the tests
// below depend on.
cookieTest('A later Path attribute overrides an earlier one', 'later',
           [DIR, `${DIR}/nomatch`], async (t, name) => {
  await setCookieViaHTTP(`${name}=1; Path=${DIR}/nomatch; Path=${DIR}`);
  assert_equals(await cookieValueAtTarget(name), '1');
});

// Control: a non-ASCII byte outside the Path attribute is not a parse failure,
// so it is specifically the path that cannot hold one.
cookieTest('A non-ASCII byte in the value is not a parse failure', 'value',
           [DIR], async (t, name) => {
  await setCookieViaHTTP(`${name}=ZZ; Path=${DIR}`, '%E4%B8%AD');
  assert_not_equals(await cookieValueAtTarget(name), null,
                    'The cookie was stored and sent');
});

// A non-ASCII byte in a Path attribute that loses to a later one does not matter,
// because only the winning Path attribute is validated.
cookieTest('A non-ASCII Path attribute followed by a valid one is not a parse failure',
           'losing', [DIR], async (t, name) => {
  await setCookieViaHTTP(`${name}=1; Path=${DIR}/zzZZ; Path=${DIR}`,
                         '%E4%B8%AD');
  assert_equals(await cookieValueAtTarget(name), '1');
});

// When the non-ASCII Path attribute is the one that wins, the cookie fails to
// parse and nothing is stored.
cookieTest('A non-ASCII byte in the winning Path attribute is a parse failure',
           'winning', [DIR], async (t, name) => {
  await setCookieViaHTTP(`${name}=1; Path=${DIR}; Path=${DIR}/zzZZ`,
                         '%E4%B8%AD');
  assert_equals(await cookieValueAtTarget(name), null);
});

// The cookie must not be sent for a path the non-ASCII bytes were appended to.
// A user agent that truncated the Path at the first non-ASCII byte, or dropped
// the attribute so that the path became "/", would send it here.
cookieTest('A Path attribute with non-ASCII bytes appended does not match that path',
           'appended', [DIR, '/'], async (t, name) => {
  await setCookieViaHTTP(`${name}=1; Path=${DIR}ZZ`, '%E4%B8%AD');
  assert_equals(await cookieValueAtTarget(name), null);
});

// Nor may it fall back to the default path, which is what an ignored Path
// attribute would produce.
cookieTest('A non-ASCII Path attribute does not fall back to the default path',
           'fallback', ['/cookies/resources'], async (t, name) => {
  await setCookieViaHTTP(`${name}=1; Path=${DIR}/zzZZ`, '%E4%B8%AD');
  assert_equals(await cookieValueAtDefaultPath(name), null);
});

// A lone non-ASCII byte is not valid UTF-8 on its own, and is equally rejected.
cookieTest('A lone non-ASCII byte in the winning Path attribute is a parse failure',
           'lone', [DIR, '/cookies/resources'], async (t, name) => {
  await setCookieViaHTTP(`${name}=1; Path=${DIR}/zzZZ`, '%B8');
  assert_equals(await cookieValueAtTarget(name), null);
  assert_equals(await cookieValueAtDefaultPath(name), null);
});
