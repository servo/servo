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

// httpCookieTest sets a |cookie| (via HTTP), then asserts it was or was not set
// via |expectedValue| (via the DOM). Then cleans it up (via HTTP). Most tests
// do not set a Path attribute, so |defaultPath| defaults to true.
function httpCookieTest(cookie, expectedValue, name, defaultPath = true) {
  let encodedCookie = encodeURIComponent(cookie);
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
