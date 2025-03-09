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

export function addCookieAndSessionCleanup(test, cookieAndAttributes) {
  // Clean up any set cookies once the test completes.
  test.add_cleanup(async () => {
    const response = await fetch('end_session_via_clear_site_data.py');
    assert_equals(response.status, 200);
    expireCookie(cookieAndAttributes);
  });
}

export async function configureServer(obj) {
  const response = await fetch('configure_server.py', {
    method: 'POST',
    headers: {'Content-Type': 'application/json'},
    body: JSON.stringify(obj),
  });
  assert_equals(response.status, 200);
}

export async function setupShardedServerState(testId) {
  const obj = {};
  if (testId !== undefined) {
    obj.testId = testId;
  }
  const response = await fetch('setup_sharded_server_state.py', {
    method: 'POST',
    headers: {'Content-Type': 'application/json'},
    body: JSON.stringify(obj),
  });
  assert_equals(response.status, 200);
  const testIdCookie =
      document.cookie.split(';').filter(item => item.includes('test_id'))[0];
  return testIdCookie.split('=')[1];
}
