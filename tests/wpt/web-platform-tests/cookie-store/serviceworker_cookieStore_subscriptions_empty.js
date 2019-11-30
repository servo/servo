self.GLOBAL = {
  isWindow: function() { return false; },
  isWorker: function() { return true; },
};
importScripts("/resources/testharness.js");

// Resolves when the service worker receives the 'activate' event.
const kServiceWorkerActivatedPromise = new Promise(resolve => {
  self.addEventListener('activate', event => { resolve(); });
});

promise_test(async testCase => {
  await kServiceWorkerActivatedPromise;

  const subscriptions = await registration.cookies.getSubscriptions();
  assert_equals(subscriptions.length, 0);

}, 'getSubscriptions returns an empty array when there are no subscriptions');

done();
