// META: title=Cookie Store API: ServiceWorker with multiple cookie change subscriptions
// META: global=window,serviceworker
// META: script=/service-workers/service-worker/resources/test-helpers.sub.js

'use strict';

// sort() comparator that uses the < operator.
//
// This is intended to be used for sorting strings. Using < is preferred to
// localeCompare() because the latter has some implementation-dependent
// behavior.
function CompareStrings(a, b) {
  return a < b ? -1 : (b < a ? 1 : 0);
}

promise_test(async testCase => {
  let scope;

  if (self.GLOBAL.isWindow()) {
    scope = '/cookie-store/resources/does/not/exist';

    const registration = await service_worker_unregister_and_register(
        testCase, 'resources/empty_sw.js', scope);
    testCase.add_cleanup(() => registration.unregister());

    // Must wait for the service worker to enter the 'activated' state before
    // subscribing to cookiechange events.
    await wait_for_state(testCase, registration.installing, 'activated');

    self.registration = registration;
  } else {
    scope = '/cookie-store/does/not/exist';

    // Must wait for the service worker to enter the 'activated' state before
    // subscribing to cookiechange events.
    await new Promise(resolve => {
      self.addEventListener('activate', event => { resolve(); });
    });
  }

  {
    const subscriptions = [{ name: 'cookie-name1', url: `${scope}/path1` }];
    await registration.cookies.subscribe(subscriptions);
    testCase.add_cleanup(() => {
      // For non-ServiceWorker environments, registration.unregister() cleans up
      // cookie subscriptions.
      if (self.GLOBAL.isWorker()) {
        return registration.cookies.unsubscribe(subscriptions);
      }
    });
  }
  {
    const subscriptions = [
      { },  // Test the default values for subscription properties.
      { name: 'cookie-prefix' },
    ];
    await registration.cookies.subscribe(subscriptions);
    testCase.add_cleanup(() => {
      // For non-ServiceWorker environments, registration.unregister() cleans up
      // cookie subscriptions.
      if (self.GLOBAL.isWorker()) {
        return registration.cookies.unsubscribe(subscriptions);
      }
    });
  }

  const subscriptions = await registration.cookies.getSubscriptions();
  assert_equals(subscriptions.length, 3);

  subscriptions.sort((a, b) => CompareStrings(`${a.name}`, `${b.name}`));

  assert_equals(subscriptions[0].name, 'cookie-name1');

  assert_equals(subscriptions[1].name, 'cookie-prefix');

  assert_false('name' in subscriptions[2]);
}, 'getSubscriptions returns a subscription passed to subscribe');
