self.GLOBAL = {
  isWindow: function() { return false; },
  isWorker: function() { return true; },
};
importScripts("/resources/testharness.js");

self.addEventListener('install', (event) => {
  event.waitUntil((async () => {
    try {
      await cookieStore.subscribeToChanges([
        { name: 'cookie-name', matchType: 'equals',
          url: '/cookie-store/scope/path' }]);

      // If the worker enters the "redundant" state, the UA may terminate it
      // before all tests have been reported to the client. Stifle errors in
      // order to avoid this and ensure all tests are consistently reported.
    } catch (err) {}
  })());
});

// Resolves when the service worker receives the 'activate' event.
const kServiceWorkerActivatedPromise = new Promise(resolve => {
  self.addEventListener('activate', event => { resolve(); });
});

promise_test(async testCase => {
  await kServiceWorkerActivatedPromise;

  const subscriptions = await cookieStore.getChangeSubscriptions();
  assert_equals(subscriptions.length, 1);

  assert_equals(subscriptions[0].name, 'cookie-name');
  assert_equals('equals', subscriptions[0].matchType);
}, 'getChangeSubscriptions returns a subscription passed to subscribeToChanges');


promise_test(async testCase => {
  await kServiceWorkerActivatedPromise;

  const cookie_change_received_promise = new Promise((resolve) => {
    self.addEventListener('cookiechange', (event) => {
      resolve(event);
    });
  });

  await cookieStore.set('cookie-name', 'cookie-value');
  testCase.add_cleanup(async () => {
    await cookieStore.delete('cookie-name');
  });

  const event = await cookie_change_received_promise;
  assert_equals(event.type, 'cookiechange');
  assert_equals(event.changed.length, 1);
  assert_equals(event.changed[0].name, 'cookie-name');
  assert_equals(event.changed[0].value, 'cookie-value');
  assert_equals(event.deleted.length, 0);
  assert_true(event instanceof ExtendableCookieChangeEvent);
  assert_true(event instanceof ExtendableEvent);
}, 'cookiechange dispatched with cookie change that matches subscription ' +
   'to event handler registered with addEventListener');

done();
