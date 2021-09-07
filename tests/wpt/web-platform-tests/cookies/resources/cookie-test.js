// getDefaultPathCookies is a helper method to get and delete cookies on the
// "default path" (which for these tests will be at `/cookies/resources`),
// determined by the path portion of the request-uri.
async function getDefaultPathCookies(path = '/cookies/resources') {
  return new Promise((resolve, reject) => {
    try {
      const iframe = document.createElement('iframe');
      iframe.style = 'display: none';
      iframe.src = `${path}/echo-cookie.html`;

      iframe.addEventListener('load', (e) => {
        const win = e.target.contentWindow;
        const iframeCookies = win.getCookies();
        win.expireCookie('test', path);
        resolve(iframeCookies);
      }, {once: true});

      document.documentElement.appendChild(iframe);
    } catch (e) {
      reject(e);
    }
  });
}

// getRedirectedCookies is a helper method to get and delete cookies that
// were set from a Location header redirect.
async function getRedirectedCookies(location, cookie) {
  return new Promise((resolve, reject) => {
    try {
      const iframe = document.createElement('iframe');
      iframe.style = 'display: none';
      iframe.src = location;

      iframe.addEventListener('load', (e) => {
        const win = e.target.contentWindow;
        let iframeCookie;
        // go ask for the cookie
        win.postMessage('getCookies', '*');

        // once we get it, send a message to delete on the other
        // side, then resolve the cookie back to httpRedirectCookieTest
        window.addEventListener('message', (e) => {
          if (typeof e.data == 'object' && 'cookies' in e.data) {
            iframeCookie = e.data.cookies;
            e.source.postMessage({'expireCookie': cookie}, '*');
          }

          // wait on the iframe to tell us it deleted the cookies before
          // resolving, to avoid any state race conditions.
          if (e.data == 'expired') {
            resolve(iframeCookie);
          }
        });
      }, {once: true});

      document.documentElement.appendChild(iframe);
    } catch (e) {
      reject(e);
    }
  });
}

// httpCookieTest sets a `cookie` (via HTTP), then asserts it was or was not set
// via `expectedValue` (via the DOM). Then cleans it up (via HTTP). Most tests
// do not set a Path attribute, so `defaultPath` defaults to true.
//
// `cookie` may be a single cookie string, or an array of cookie strings, where
// the order of the array items represents the order of the Set-Cookie headers
// sent by the server.
function httpCookieTest(cookie, expectedValue, name, defaultPath = true) {
  return promise_test(async (t) => {
    let encodedCookie = encodeURIComponent(JSON.stringify(cookie));
    await fetch(`/cookies/resources/cookie.py?set=${encodedCookie}`);
    let cookies = document.cookie;
    if (defaultPath) {
      // for the tests where a Path is set from the request-uri
      // path, we need to go look for cookies in an iframe at that
      // default path.
      cookies = await getDefaultPathCookies();
    }
    if (Boolean(expectedValue)) {
      assert_equals(cookies, expectedValue, 'The cookie was set as expected.');
    } else {
      assert_equals(cookies, expectedValue, 'The cookie was rejected.');
    }
    await fetch(`/cookies/resources/cookie.py?drop=${encodedCookie}`);
  }, name);
}

// This is a variation on httpCookieTest, where a redirect happens via
// the Location header and we check to see if cookies are sent via
// getRedirectedCookies
function httpRedirectCookieTest(cookie, expectedValue, name, location) {
  return promise_test(async (t) => {
    const encodedCookie = encodeURIComponent(JSON.stringify(cookie));
    const encodedLocation = encodeURIComponent(location);
    const setParams = `?set=${encodedCookie}&location=${encodedLocation}`;
    await fetch(`/cookies/resources/cookie.py${setParams}`);
    // for the tests where a redirect happens, we need to head
    // to that URI to get the cookies (and then delete them there)
    const cookies = await getRedirectedCookies(location, cookie);
    if (Boolean(expectedValue)) {
      assert_equals(cookies, expectedValue, 'The cookie was set as expected.');
    } else {
      assert_equals(cookies, expectedValue, 'The cookie was rejected.');
    }
    await fetch(`/cookies/resources/cookie.py?drop=${encodedCookie}`);
  }, name);
}

// Sets a `cookie` via the DOM, checks it against `expectedValue` via the DOM,
// then cleans it up via the DOM. This is needed in cases where going through
// HTTP headers may modify the cookie line (e.g. by stripping control
// characters).
//
// Note: this function has a dependency on testdriver.js. Any test files calling
// it should include testdriver.js and testdriver-vendor.js
function domCookieTest(cookie, expectedValue, name) {
  return promise_test(async (t) => {
    await test_driver.delete_all_cookies();
    t.add_cleanup(test_driver.delete_all_cookies);

    document.cookie = cookie;
    let cookies = document.cookie;
    assert_equals(cookies, expectedValue, Boolean(expectedValue) ?
                                          'The cookie was set as expected.' :
                                          'The cookie was rejected.');
  }, name);
}

// Returns two arrays of control characters along with their ASCII codes. The
// TERMINATING_CTLS should result in termination of the cookie string. The
// remaining CTLS should result in rejection of the cookie. Control characters
// are defined by RFC 5234 to be %x00-1F / %x7F.
function getCtlCharacters() {
  const termCtlCodes = [0x00 /* NUL */, 0x0A /* LF */, 0x0D /* CR */];
  const ctlCodes = [...Array(0x20).keys()]
                       .filter(i => termCtlCodes.indexOf(i) === -1)
                       .concat([0x7F]);
  return {
    TERMINATING_CTLS:
        termCtlCodes.map(i => ({code: i, chr: String.fromCharCode(i)})),
    CTLS: ctlCodes.map(i => ({code: i, chr: String.fromCharCode(i)}))
  };
}

// Returns a cookie string with name set to "t" * nameLength and value
// set to "1" * valueLength. Passing in 0 for either allows for creating
// a name- or value-less cookie.
//
// Note: Cookie length checking should ignore the "=".
function cookieStringWithNameAndValueLengths(nameLength, valueLength) {
  return `${"t".repeat(nameLength)}=${"1".repeat(valueLength)}`;
}
