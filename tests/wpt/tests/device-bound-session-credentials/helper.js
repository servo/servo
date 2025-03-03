export function documentHasCookie(cookieAndValue) {
  return document.cookie.split(';').some(item => item.includes(cookieAndValue));
}

export function waitForCookie(cookieAndValue) {
  const startTime = Date.now();
  return new Promise(resolve => {
    const interval = setInterval(() => {
      if (documentHasCookie(cookieAndValue)) {
        clearInterval(interval);
        resolve(true);
      }
      if (Date.now() - startTime >= 1000) {
        clearInterval(interval);
        resolve(false);
      }
    }, 100);
  });
}

export function expireCookie(cookieAndAttributes) {
  document.cookie =
      cookieAndAttributes + '; expires=Thu, 01 Jan 1970 00:00:00 UTC;';
}

export function addCookieAndServerCleanup(test, cookieAndAttributes) {
  // Clean up any set cookies once the test completes.
  test.add_cleanup(async () => {
    const response = await fetch('clear_server_state_and_end_sessions.py');
    assert_equals(response.status, 200);
    expireCookie(cookieAndAttributes);
  });
}
