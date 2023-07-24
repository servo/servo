// META: title=Cookie Store API: oncookiechange event in ServiceWorker with single subscription
// META: global=serviceworker

'use strict';

const kScope = '/cookie-store/does/not/exist';

// Resolves when the service worker receives the 'activate' event.
const kServiceWorkerActivatedPromise = new Promise((resolve) => {
  self.addEventListener('activate', event => { resolve(); });
});

// Resolves when a cookiechange event is received.
const kCookieChangeReceivedPromise = new Promise(resolve => {
  self.oncookiechange = event => { resolve(event); };
});

promise_test(async testCase => {
  await kServiceWorkerActivatedPromise;

  const subscriptions = [{ name: 'cookie-name', url: `${kScope}/path` }];
  await registration.cookies.subscribe(subscriptions);
  testCase.add_cleanup(() => registration.cookies.unsubscribe(subscriptions));

  await cookieStore.set('cookie-name', 'cookie-value');
  testCase.add_cleanup(async () => {
    await cookieStore.delete('cookie-name');
  });

  const event = await kCookieChangeReceivedPromise;
  assert_equals(event.type, 'cookiechange');
  assert_equals(event.changed.length, 1);
  assert_equals(event.changed[0].name, 'cookie-name');
  assert_equals(event.changed[0].value, 'cookie-value');
  assert_equals(event.deleted.length, 0);
  assert_true(event instanceof ExtendableCookieChangeEvent);
  assert_true(event instanceof ExtendableEvent);
}, 'cookiechange dispatched with cookie change that matches subscription ' +
   'to cookiechange event handler registered with addEventListener');
