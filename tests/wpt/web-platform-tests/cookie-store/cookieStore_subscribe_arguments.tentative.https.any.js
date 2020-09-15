// META: title=Cookie Store API: cookieStore.subscribe() arguments
// META: global=window,serviceworker
// META: script=/service-workers/service-worker/resources/test-helpers.sub.js

'use strict';

promise_test(async testCase => {
  if (self.GLOBAL.isWindow()) {
    const registration = await service_worker_unregister_and_register(
        testCase, 'resources/empty_sw.js',
        '/cookie-store/resources/does/not/exist');
    testCase.add_cleanup(() => registration.unregister());

    // Must wait for the service worker to enter the 'activated' state before
    // subscribing to cookiechange events.
    await wait_for_state(testCase, registration.installing, 'activated');

    self.registration = registration;
  } else {
    // Must wait for the service worker to enter the 'activated' state before
    // subscribing to cookiechange events.
    await new Promise(resolve => {
      self.addEventListener('activate', event => { resolve(); });
    });
  }

  {
    const subscriptions = [{ name: 'cookie-name' }];
    await self.registration.cookies.subscribe(subscriptions);
    testCase.add_cleanup(() => registration.cookies.unsubscribe(subscriptions));
  }

  const subscriptions = await registration.cookies.getSubscriptions();
  assert_equals(subscriptions.length, 1);

  assert_equals(subscriptions[0].name, 'cookie-name');
  assert_equals(subscriptions[0].url, registration.scope);
}, 'cookieStore.subscribe without url in option');

promise_test(async testCase => {
  if (self.GLOBAL.isWindow()) {
    const registration = await service_worker_unregister_and_register(
        testCase, 'resources/empty_sw.js',
        '/cookie-store/resources/does/not/exist');
    testCase.add_cleanup(() => registration.unregister());

    // Must wait for the service worker to enter the 'activated' state before
    // subscribing to cookiechange events.
    await wait_for_state(testCase, registration.installing, 'activated');

    self.registration = registration;
  } else if (!self.registration.active) {
    // If service worker is not active yet, it must wait for it to enter the
    // 'activated' state before subscribing to cookiechange events.
    await new Promise(resolve => {
      self.addEventListener('activate', event => { resolve(); });
    });
  }

  await promise_rejects_js(testCase, TypeError,
      registration.cookies.subscribe(
          { name: 'cookie-name', url: '/wrong/path' }));
}, 'cookieStore.subscribe with invalid url path in option');

promise_test(async testCase => {
  if (self.GLOBAL.isWindow()) {
    const registration = await service_worker_unregister_and_register(
        testCase, 'resources/empty_sw.js',
        '/cookie-store/resources/does/not/exist');
    testCase.add_cleanup(() => registration.unregister());

    // Must wait for the service worker to enter the 'activated' state before
    // subscribing to cookiechange events.
    await wait_for_state(testCase, registration.installing, 'activated');

    self.registration = registration;
  } else if (!self.registration.active) {
    // Must wait for the service worker to enter the 'activated' state before
    // subscribing to cookiechange events.
    await new Promise(resolve => {
      self.addEventListener('activate', event => { resolve(); });
    });
  }

  {
    const subscriptions = [{ name: 'cookie-name' }];
    // Call subscribe for same subscription multiple times to verify that it is
    // idempotent.
    await self.registration.cookies.subscribe(subscriptions);
    await self.registration.cookies.subscribe(subscriptions);
    await self.registration.cookies.subscribe(subscriptions);
    testCase.add_cleanup(() => registration.cookies.unsubscribe(subscriptions));
  }

  const subscriptions = await registration.cookies.getSubscriptions();
  assert_equals(subscriptions.length, 1);

  assert_equals(subscriptions[0].name, 'cookie-name');
  assert_equals(subscriptions[0].url, registration.scope);
}, 'cookieStore.subscribe is idempotent');

promise_test(async testCase => {
  if (self.GLOBAL.isWindow()) {
    const registration = await service_worker_unregister_and_register(
        testCase, 'resources/empty_sw.js',
        '/cookie-store/resources/does/not/exist');
    testCase.add_cleanup(() => registration.unregister());

    // Must wait for the service worker to enter the 'activated' state before
    // subscribing to cookiechange events.
    await wait_for_state(testCase, registration.installing, 'activated');

    self.registration = registration;
  } else if (!self.registration.active) {
    // Must wait for the service worker to enter the 'activated' state before
    // subscribing to cookiechange events.
    await new Promise(resolve => {
      self.addEventListener('activate', event => { resolve(); });
    });
  }

  {
    const subscriptions = [
      { name: 'cookie-name1' },
      { name: 'cookie-name2' },
    ];
    await self.registration.cookies.subscribe(subscriptions);
    testCase.add_cleanup(() => registration.cookies.unsubscribe(subscriptions));

    // Call unsubscribe for same subscription multiple times to verify that it
    // is idempotent.
    await registration.cookies.unsubscribe([subscriptions[0]]);
    await registration.cookies.unsubscribe([subscriptions[0]]);
    await registration.cookies.unsubscribe([subscriptions[0]]);
  }

  const subscriptions = await registration.cookies.getSubscriptions();
  assert_equals(subscriptions.length, 1);

  assert_equals(subscriptions[0].name, 'cookie-name2');
  assert_equals(subscriptions[0].url, registration.scope);
}, 'CookieStore.unsubscribe is idempotent');
