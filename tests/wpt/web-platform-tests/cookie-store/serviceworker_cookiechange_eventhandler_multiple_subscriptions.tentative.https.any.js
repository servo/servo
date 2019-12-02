// META: title=Cookie Store API: cookiechange event in ServiceWorker with multiple subscriptions
// META: global=!default,serviceworker

'use strict';

const kScope = '/cookie-store/does/not/exist';

// Resolves when the service worker receives the 'activate' event.
const kServiceWorkerActivatedPromise = new Promise((resolve) => {
  self.addEventListener('activate', event => { resolve(); });
});

// Accumulates cookiechange events dispatched to the service worker.
let g_cookie_changes = [];

// Resolved when a cookiechange event is received. Rearmed by
// RearmCookieChangeReceivedPromise().
let g_cookie_change_received_promise = null;
let g_cookie_change_received_promise_resolver = null;
self.addEventListener('cookiechange', (event) => {
  g_cookie_changes.push(event);
  if (g_cookie_change_received_promise_resolver) {
    g_cookie_change_received_promise_resolver();
  }
});
function RearmCookieChangeReceivedPromise() {
  g_cookie_change_received_promise = new Promise((resolve) => {
    g_cookie_change_received_promise_resolver = resolve;
  });
}
RearmCookieChangeReceivedPromise();

promise_test(async testCase => {
  await kServiceWorkerActivatedPromise;

  {
    const subscriptions = [
      { name: 'cookie-name1', matchType: 'equals', url: `${kScope}/path1` },
    ];
    await registration.cookies.subscribe(subscriptions);
    testCase.add_cleanup(() => registration.cookies.unsubscribe(subscriptions));
  }
  {
    const subscriptions = [
      { },  // Test the default values for subscription properties.
      { name: 'cookie-prefix', matchType: 'starts-with' },
    ];
    await registration.cookies.subscribe(subscriptions);
    testCase.add_cleanup(() => registration.cookies.unsubscribe(subscriptions));
  }

  await cookieStore.set('cookie-name', 'cookie-value');
  testCase.add_cleanup(async () => {
    await cookieStore.delete('cookie-name');
  });
  testCase.add_cleanup(() => { g_cookie_changes = []; });

  await g_cookie_change_received_promise;
  testCase.add_cleanup(() => RearmCookieChangeReceivedPromise());

  assert_equals(g_cookie_changes.length, 1);
  const event = g_cookie_changes[0];
  assert_equals(event.type, 'cookiechange');
  assert_equals(event.changed.length, 1);
  assert_equals(event.changed[0].name, 'cookie-name');
  assert_equals(event.changed[0].value, 'cookie-value');
  assert_equals(event.deleted.length, 0);
  assert_true(event instanceof ExtendableCookieChangeEvent);
  assert_true(event instanceof ExtendableEvent);
}, 'cookiechange dispatched with cookie change that matches subscription');
