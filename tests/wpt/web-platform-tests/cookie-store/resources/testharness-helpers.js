'use strict';

// Length of final setTimeout when observer callback has not fired.
//
const kExtraObserverDelay = 0;  // For builtin implementation

// NOTE: A polyfill was used for pre-implementation testing. To revive
// it uncomment these and comment out the preceding line:
//
// const kExtraObserverDelay = 200;  // Polyfill when not running on battery
// // const kExtraObserverDelay = 5000;  // ... when running on battery
// document.open();
// document.write(`
//   <script>delete cookieStore</script>
//   <script src="https://wicg.github.io/cookie-store/cookies.js">
//   </script>
// `);
// document.close()

// See https://github.com/whatwg/html/pull/3011#issuecomment-331187136
// and https://www.chromestatus.com/feature/6170540112871424
const kMetaHttpEquivSetCookieIsGone = true;

// True when running in a document context as opposed to a worker context
const kHasDocument = typeof document !== 'undefined';

// Override for named test inclusion. Set by suite().
let testOverride = undefined;

// Determines whether the named test should be included in this run of the
// suite. Only usable in a test runner context as this uses assert_equals.
//
// Parameters:
//
// - testName: (string) test name; must be an identifier starting with 'test'
// - opt_excludeFromAll: (optional; boolean) if true, explicit or implicit
//   #...&test=all (which is the default) will not activate this test.
const includeTest = (testName, opt_excludeFromAll) => {
  assert_equals(!!testName.match(/^test\w+/), true, 'includeTest: ' + testName);
  assert_equals(typeof eval(testName), 'function', 'includeTest: ' + testName);
  let testParams =
        (location.hash || '#').substr(1).split('&').filter(
            x => x.match(/^test=/)).map(x => decodeURIComponent(x));
  if (!testParams.length) {
    testParams = ['test=all'];
    if (testOverride !== undefined) {
      testParams = ['test=' + testOverride];
    }
  }
  const filterSet =
        testParams.map(x => x.split('=', 2)[1]).join(',').split(',').reduce(
            (set, name) => Object.assign(set, {[name]: true}), {});
  for (let name in filterSet) {
    if (name === 'all' || !filterSet.hasOwnProperty(name)) continue;
    assert_equals(!!name.match(/^test\w+/), true, '#test=' + testName);
    assert_equals(typeof eval(name), 'function', '#test=' + testName);
  }
  return (filterSet.all && !opt_excludeFromAll) ||
      filterSet.hasOwnProperty(testName) && filterSet[testName];
}

// True when running on unsecured 'http:' rather than secured 'https:'.
const kIsUnsecured = location.protocol !== 'https:';

// True when no CGI/no active wptserve handlers should be used.
const kIsStatic = !!((location.hash || '#').match(/(^#|&)static=true(&|$)/) ||
                     location.pathname.match(/_static\./));

const kCookieHelperCgi = 'resources/cookie_helper.py';

// Async wrapper for an async function or promise that is expected
// reject in an unsecured (non-https:) context and work in a secured
// (https:) context.
//
// Parameters:
//
// - testCase: (TestCase) test case context
// - code: (Error class or number) expected rejection type in unsecured context
// - promise: (thenable) test code
// - message: (optional; string) message to forward to promise_rejects in
//   unsecured context
const promise_rejects_when_unsecured = async (
    testCase,
    code,
    promise,
    message = 'Feature unavailable from unsecured contexts'
) => {
  if (kIsUnsecured) await promise_rejects(testCase, code, promise, message);
  else await promise;
};

// Converts a list of cookie records {name, value} to [name=]value; ... as
// seen in Cookie: and document.cookie.
//
// Parameters:
// - cookies: (array of {name, value}) records to convert
//
// Returns a string serializing the records, or undefined if no records were
// given.
const cookieString = cookies => cookies.length ? cookies.map((
    {name, value}) => (name ? (name + '=') : '') + value).join('; ') :
      undefined;

// Approximate async equivalent to the document.cookie getter but with
// important differences: optional additional getAll arguments are
// forwarded, and an empty cookie jar returns undefined.
//
// This is intended primarily for verification against expected cookie
// jar contents. It should produce more readable messages using
// assert_equals in failing cases than assert_object_equals would
// using parsed cookie jar contents and also allows expectations to be
// written more compactly.
const getCookieString = async (...args) => {
  return cookieString(await cookieStore.getAll(...args));
}

// Approximate async equivalent to the document.cookie getter but from
// the server's point of view. Returns UTF-8 interpretation. Allows
// sub-path to be specified.
//
// Unlike document.cookie, this returns undefined when no cookies are
// present.
const getCookieStringHttp = async (extraPath = null) => {
  if (kIsStatic) throw 'CGI not available in static HTML test';
  const url =
        kCookieHelperCgi + ((extraPath == null) ? '' : ('/' + extraPath));
  const response = await fetch(url, { credentials: 'include' });
  const text = await response.text();
  assert_equals(
      response.ok,
      true,
      'CGI should have succeeded in getCookieStringHttp\n' + text);
  assert_equals(
      response.headers.get('content-type'),
      'text/plain; charset=utf-8',
      'CGI did not return UTF-8 text in getCookieStringHttp');
  if (text === '') return undefined;
  assert_equals(
      text.indexOf('cookie='),
      0,
      'CGI response did not begin with "cookie=" and was not empty: ' + text);
  return decodeURIComponent(text.replace(/^cookie=/, ''));
}

// Approximate async equivalent to the document.cookie getter but from
// the server's point of view. Returns binary string
// interpretation. Allows sub-path to be specified.
//
// Unlike document.cookie, this returns undefined when no cookies are
// present.
const getCookieBinaryHttp = async (extraPath = null) => {
  if (kIsStatic) throw 'CGI not available in static HTML test';
  const url =
        kCookieHelperCgi +
        ((extraPath == null) ?
         '' :
         ('/' + extraPath)) + '?charset=iso-8859-1';
  const response = await fetch(url, { credentials: 'include' });
  const text = await response.text();
  assert_equals(
      response.ok,
      true,
      'CGI should have succeeded in getCookieBinaryHttp\n' + text);
  assert_equals(
      response.headers.get('content-type'),
      'text/plain; charset=iso-8859-1',
      'CGI did not return ISO 8859-1 text in getCookieBinaryHttp');
  if (text === '') return undefined;
  assert_equals(
      text.indexOf('cookie='),
      0,
      'CGI response did not begin with "cookie=" and was not empty: ' + text);
  return unescape(text.replace(/^cookie=/, ''));
}

// Approximate async equivalent to the document.cookie setter but from
// the server's point of view.
const setCookieStringHttp = async setCookie => {
  if (kIsStatic) throw 'CGI not available in static HTML test';
  const encodedSetCookie = encodeURIComponent(setCookie);
  const url = kCookieHelperCgi;
  const headers = new Headers();
  headers.set(
      'content-type',
      'application/x-www-form-urlencoded; charset=utf-8');
  const response = await fetch(
      url,
      {
        credentials: 'include',
        method: 'POST',
        headers: headers,
        body: 'set-cookie=' + encodedSetCookie,
      });
  const text = await response.text();
  assert_equals(
      response.ok,
      true,
      'CGI should have succeeded in setCookieStringHttp set-cookie: ' +
        setCookie + '\n' + text);
  assert_equals(
      response.headers.get('content-type'),
      'text/plain; charset=utf-8',
      'CGI did not return UTF-8 text in setCookieStringHttp');
  assert_equals(
      text,
      'set-cookie=' + encodedSetCookie,
      'CGI did not faithfully echo the set-cookie value');
};

// Approximate async equivalent to the document.cookie setter but from
// the server's point of view. This version sets a binary cookie rather
// than a UTF-8 one.
const setCookieBinaryHttp = async setCookie => {
  if (kIsStatic) throw 'CGI not available in static HTML test';
  const encodedSetCookie = escape(setCookie).split('/').join('%2F');
  const url = kCookieHelperCgi + '?charset=iso-8859-1';
  const headers = new Headers();
  headers.set(
      'content-type',
      'application/x-www-form-urlencoded; charset=iso-8859-1');
  const response = await fetch(url, {
    credentials: 'include',
    method: 'POST',
    headers: headers,
    body: 'set-cookie=' + encodedSetCookie
  });
  const text = await response.text();
  assert_equals(
      response.ok,
      true,
      'CGI should have succeeded in setCookieBinaryHttp set-cookie: ' +
        setCookie + '\n' + text);
  assert_equals(
      response.headers.get('content-type'),
      'text/plain; charset=iso-8859-1',
      'CGI did not return Latin-1 text in setCookieBinaryHttp');
  assert_equals(
      text,
      'set-cookie=' + encodedSetCookie,
      'CGI did not faithfully echo the set-cookie value');
};

// Approximate async equivalent to the document.cookie setter but using
// <meta http-equiv="set-cookie" content="..."> written into a temporary
// IFRAME. Merely appending the node to HEAD works in some browsers (e.g.
// Chromium) but not others (e.g. Firefox.)
const setCookieStringMeta = async setCookie => {
  if (document.readyState !== 'complete') {
    await new Promise(resolve => addEventListener('load', resolve, true));
  }
  const meta = Object.assign(document.createElement('meta'), {
    httpEquiv: 'set-cookie',
    content: setCookie
  });
  const ifr = document.createElement('iframe');
  await new Promise(resolve => document.body.appendChild(Object.assign(
      ifr,
      {
        onload: resolve
      })));
  try {
    ifr.contentWindow.document.open('text/html; charset=utf-8');
    ifr.contentWindow.document.write([
      '<!DOCTYPE html>',
      '<meta charset="utf-8">',
      meta.outerHTML
    ].join('\r\n'));
    ifr.contentWindow.document.close();
  } finally {
    if (ifr.parentNode) ifr.parentNode.removeChild(ifr);
  }
};

// Async document.cookie getter; converts '' to undefined which loses
// information in the edge case where a single ''-valued anonymous
// cookie is visible.
const getCookieStringDocument = async () => {
  if (!kHasDocument) throw 'document.cookie not available in this context';
  return String(document.cookie || '') || undefined;
};

// Async document.cookie setter
const setCookieStringDocument = async setCookie => {
  if (!kHasDocument) throw 'document.cookie not available in this context';
  document.cookie = setCookie;
};
