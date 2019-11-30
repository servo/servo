self.GLOBAL = {
  isWindow: function() { return false; },
  isWorker: function() { return true; },
};
importScripts("/resources/testharness.js");

// Resolves when the service worker receives the 'activate' event.
const kServiceWorkerActivatedPromise = new Promise(resolve => {
  self.addEventListener('activate', event => { resolve(); });
});

// sort() comparator that uses the < operator.
//
// This is intended to be used for sorting strings. Using < is preferred to
// localeCompare() because the latter has some implementation-dependent
// behavior.
function CompareStrings(a, b) {
  return a < b ? -1 : (b < a ? 1 : 0);
}

promise_test(async testCase => {
  await kServiceWorkerActivatedPromise;

  {
    const subscriptions = [
      { name: 'cookie-name1', matchType: 'equals', url: '/cookie-store/path1' },
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

  const subscriptions = await registration.cookies.getSubscriptions();
  assert_equals(subscriptions.length, 3);

  subscriptions.sort((a, b) => CompareStrings(`${a.name}`, `${b.name}`));

  assert_equals(subscriptions[0].name, 'cookie-name1');
  assert_equals('equals', subscriptions[0].matchType);

  assert_equals(subscriptions[1].name, 'cookie-prefix');
  assert_equals('starts-with', subscriptions[1].matchType);

  assert_false('name' in subscriptions[2]);
  assert_equals('starts-with', subscriptions[2].matchType);
}, 'getSubscriptions returns subscriptions passed to subscribe');

// Accumulates cookiechange events dispatched to the service worker.
let g_cookie_changes = [];

// Resolved when a cookiechange event is received. Rearmed by
// RearmCookieChangeReceivedPromise().
let g_cookie_change_received_promise = null;
let g_cookie_change_received_promise_resolver = null;
self.addEventListener('cookiechange', (event) => {
  g_cookie_changes.push(event);
  if (g_cookie_change_received_promise_resolver)
    g_cookie_change_received_promise_resolver();
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
      { name: 'cookie-name1', matchType: 'equals', url: '/cookie-store/path1' },
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
  const event = g_cookie_changes[0]
  assert_equals(event.type, 'cookiechange');
  assert_equals(event.changed.length, 1);
  assert_equals(event.changed[0].name, 'cookie-name');
  assert_equals(event.changed[0].value, 'cookie-value');
  assert_equals(event.deleted.length, 0);
  assert_true(event instanceof ExtendableCookieChangeEvent);
  assert_true(event instanceof ExtendableEvent);
}, 'cookiechange dispatched with cookie change that matches subscription');

done();
