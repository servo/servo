self.GLOBAL = {
  isWindow: function() { return false; },
  isWorker: function() { return true; },
};
importScripts("/resources/testharness.js");

self.addEventListener('install', (event) => {
  event.waitUntil((async () => {
    cookieStore.subscribeToChanges([
      { name: 'cookie-name', matchType: 'equals', url: '/scope/path' }]);
  })());
});

// Workaround because add_cleanup doesn't support async functions yet.
// See https://github.com/w3c/web-platform-tests/issues/6075
async function async_cleanup(cleanup_function) {
  try {
    await cleanup_function();
  } catch (e) {
    // Errors in cleanup functions shouldn't result in test failures.
  }
}

// Resolves when the service worker receives the 'activate' event.
const kServiceWorkerActivatedPromise = new Promise(resolve => {
  self.addEventListener('activate', event => { resolve(); });
});

promise_test(async testCase => {
  await kServiceWorkerActivatedPromise;

  const cookie_change_received_promise = new Promise((resolve) => {
    self.addEventListener('cookiechange', (event) => {
      resolve(event);
    });
  });

  await cookieStore.set('another-cookie-name', 'cookie-value');
  await cookieStore.set('cookie-name', 'cookie-value');

  const event = await cookie_change_received_promise;
  assert_equals(event.type, 'cookiechange');
  assert_equals(event.changed.length, 1);
  assert_equals(event.changed[0].name, 'cookie-name');
  assert_equals(event.changed[0].value, 'cookie-value');

  await async_cleanup(() => {
    cookieStore.delete('another-cookie-name');
    cookieStore.delete('cookie-name');
  });
}, 'cookiechange not dispatched for change that does not match subscription');

done();
