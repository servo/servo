// META: title=Cookie Store API: ServiceWorker with one cookie change subscription
// META: global=!default,serviceworker,window
// META: script=/service-workers/service-worker/resources/test-helpers.sub.js

'use strict';

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
    const subscriptions = [
      { name: 'cookie-name', matchType: 'equals', url: `${scope}/path` }
    ];
    await registration.cookies.subscribe(subscriptions);
    testCase.add_cleanup(() => registration.cookies.unsubscribe(subscriptions));
  }

  const subscriptions = await registration.cookies.getSubscriptions();
  assert_equals(subscriptions.length, 1);

  assert_equals(subscriptions[0].name, 'cookie-name');
  assert_equals(subscriptions[0].matchType, 'equals');
  assert_equals(subscriptions[0].url,
                (new URL(`${scope}/path`, self.location.href)).href);
}, 'getSubscriptions returns a subscription passed to subscribe');
