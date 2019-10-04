self.GLOBAL = {
  isWindow: function() { return false; },
  isWorker: function() { return true; },
};
importScripts("/resources/testharness.js");

self.addEventListener('install', event => {
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

self.addEventListener('message', async event => {
  const subscriptions = await cookieStore.getChangeSubscriptions();
  event.ports[0].postMessage(subscriptions.length);
});