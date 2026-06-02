export function documentHasCookie(cookieAndValue) {
  return document.cookie.split(';').some(item => item.includes(cookieAndValue));
}

export async function waitForCookie(cookieAndValue, expectCookie) {
  const startTime = Date.now();
  const hasCookie = await new Promise(resolve => {
    const interval = setInterval(() => {
      if (documentHasCookie(cookieAndValue)) {
        clearInterval(interval);
        resolve(true);
      }
      if (!expectCookie && Date.now() - startTime >= 1000) {
        clearInterval(interval);
        resolve(false);
      }
    }, 100);
  });
  assert_equals(hasCookie, expectCookie);
}

export function expireCookie(cookieAndAttributes) {
  document.cookie =
      `${cookieAndAttributes}; expires=Thu, 01 Jan 1970 00:00:00 UTC;`;
}

export function addCookieAndSessionCleanup(test) {
  // Clean up any set cookies once the test completes.
  test.add_cleanup(async () => {
    const response = await fetch('end_session_via_clear_site_data.py');
    assert_equals(response.status, 200);
  });
}

export async function postJson(url, obj) {
  return await fetch(url, {
    method: 'POST',
    headers: {'Content-Type': 'application/json'},
    body: JSON.stringify(obj),
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

export async function setupShardedServerState(obj) {
  if (obj === undefined) {
    obj = {};
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

export async function pullServerState() {
  const response = await fetch('pull_server_state.py');
  assert_equals(response.status, 200);
  return await response.json();
}

// Create an iframe that fetches URLs on demand via postMessage.
export async function crossSiteFetch(fromSite, url, fetchParams) {
  const frame = document.createElement('iframe');
  const frameLoadPromise = new Promise((resolve, reject) => {
    frame.onload = resolve;
    frame.onerror = reject;
  });
  frame.src = fromSite + "/device-bound-session-credentials/url_fetcher.html";
  document.body.appendChild(frame);
  await frameLoadPromise;

  const fetchStatusPromise = new Promise((resolve) => {
    const listener = (event) => {
      window.removeEventListener("message", listener);
      resolve(event.data);
    };
    window.addEventListener("message", listener);
  });
  frame.contentWindow.postMessage({url, fetchParams}, "*");

  const fetchStatus = await fetchStatusPromise;
  document.body.removeChild(frame);

  return fetchStatus;
}
