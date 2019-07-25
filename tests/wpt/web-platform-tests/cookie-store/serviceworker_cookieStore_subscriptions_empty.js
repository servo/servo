self.GLOBAL = {
  isWindow: function() { return false; },
  isWorker: function() { return true; },
};
importScripts("/resources/testharness.js");

self.addEventListener('install', (event) => {
  event.waitUntil((async () => {
    try {
      await cookieStore.subscribeToChanges([]);

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
  assert_equals(subscriptions.length, 0);

}, 'getChangeSubscriptions returns an empty array when there are no subscriptions');

done();
