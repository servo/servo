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

// httpCookieTest sets a |cookie| (via HTTP), then asserts it was or was not set
// via |expectedValue| (via the DOM). Then cleans it up (via HTTP). Most tests
// do not set a Path attribute, so |defaultPath| defaults to true.
//
// |cookie| may be a single cookie string, or an array of cookie strings, where
// the order of the array items represents the order of the Set-Cookie headers
// sent by the server.
function httpCookieTest(cookie, expectedValue, name, defaultPath = true) {
  let encodedCookie = encodeURIComponent(JSON.stringify(cookie));
  return promise_test(
      async t => {
          return fetch(`/cookies/resources/cookie.py?set=${encodedCookie}`)
              .then(async () => {
                let cookies = document.cookie;
                if (defaultPath) {
                  // for the tests where a Path is set from the request-uri
                  // path, we need to go look for cookies in an iframe at that
                  // default path.
                  cookies = await getDefaultPathCookies();
                }
                if (Boolean(expectedValue)) {
                  assert_equals(
                      cookies, expectedValue,
                      'The cookie was set as expected.');
                } else {
                  assert_equals(
                      cookies, expectedValue, 'The cookie was rejected.');
                }
              })
              .then(() => {
                return fetch(
                    `/cookies/resources/cookie.py?drop=${encodedCookie}`);
              })},
      name);
}

// This is a variation on httpCookieTest, where a redirect happens via
// the Location header and we check to see if cookies are sent via
// getRedirectedCookies
function httpRedirectCookieTest(cookie, expectedValue, name, location) {
  const encodedCookie = encodeURIComponent(JSON.stringify(cookie));
  const encodedLocation = encodeURIComponent(location);
  const setParams = `?set=${encodedCookie}&location=${encodedLocation}`;
  return promise_test(
    async t => {
      return fetch(`/cookies/resources/cookie.py${setParams}`)
        .then(async () => {
          // for the tests where a redirect happens, we need to head
          // to that URI to get the cookies (and then delete them there)
          const cookies = await getRedirectedCookies(location, cookie);
          if (Boolean(expectedValue)) {
            assert_equals(cookies, expectedValue,
                          'The cookie was set as expected.');
          } else {
            assert_equals(cookies, expectedValue, 'The cookie was rejected.');
          }
        }).then(() => {
          return fetch(`/cookies/resources/cookie.py?drop=${encodedCookie}`);
        })
    },
    name);
}

// Sets a `cookie` via the DOM, checks it against `expectedValue` via the DOM,
// then cleans it up via the DOM. This is needed in cases where going through
// HTTP headers may modify the cookie line (e.g. by stripping control
// characters).
function domCookieTest(cookie, expectedValue, name) {
  return test(() => {
    document.cookie = cookie;
    let cookies = document.cookie;
    if (Boolean(expectedValue)) {
      assert_equals(cookies, expectedValue, 'The cookie was set as expected.');
    } else {
      assert_equals(cookies, expectedValue, 'The cookie was rejected.');
    }
    document.cookie = `${expectedValue}; expires=01 Jan 1970 00:00:00 GMT`;
    assert_equals(
        document.cookie, '', 'The cookies were cleaned up properly post-test.');
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
