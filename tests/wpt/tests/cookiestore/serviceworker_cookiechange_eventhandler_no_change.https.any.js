// META: title=Cookie Store API: cookiechange event in ServiceWorker with already-expired cookie.
// META: global=serviceworker

'use strict';

const kScope = '/cookiestore/does/not/exist';

// Resolves when the service worker receives the 'activate' event.
function WorkerActivationPromise() {
  return new Promise((resolve) => {
    if (registration.active) {
      resolve();
      return;
    }
    self.addEventListener('activate', () => { resolve(); });
  });
}

// Resolves when a cookiechange event is received.
function RunOnceCookieChangeReceivedPromise() {
  return new Promise(resolve => {
    const listener = ev => {
      resolve(ev);
      self.removeEventListener('cookiechange', listener);
    };
    self.addEventListener('cookiechange', listener);
  });
}

promise_test(async t => {
  await WorkerActivationPromise();

  const subscriptions = [{url: `${kScope}/path`}];
  await registration.cookies.subscribe(subscriptions);
  t.add_cleanup(() => registration.cookies.unsubscribe(subscriptions));

  let cookie_change_promise = RunOnceCookieChangeReceivedPromise();

  await cookieStore.set('cookie-name', 'value');
  t.add_cleanup(async () => {
    await cookieStore.delete('cookie-name');
  });

  // Observes original cookie.
  let event = await cookie_change_promise;
  assert_equals(event.type, 'cookiechange');
  assert_equals(event.changed.length, 1);
  assert_equals(event.changed[0].name, 'cookie-name');
  assert_equals(event.changed[0].value, 'value');
  assert_equals(event.deleted.length, 0);

  cookie_change_promise = RunOnceCookieChangeReceivedPromise();

  // Duplicate overwrite should not be observed.
  await cookieStore.set('cookie-name', 'value');

  // This cookie should be observed instead.
  await cookieStore.set('alternate-cookie-name', 'ignore');
  t.add_cleanup(async () => {
    await cookieStore.delete('alternate-cookie-name');
  });

  event = await cookie_change_promise;
  assert_equals(event.type, 'cookiechange');
  assert_equals(event.changed.length, 1);
  assert_equals(event.changed[0].name, 'alternate-cookie-name');
  assert_equals(event.changed[0].value, 'ignore');
  assert_equals(event.deleted.length, 0);
});

promise_test(async t => {
  await WorkerActivationPromise();

  const subscriptions = [{url: `${kScope}/path`}];
  await registration.cookies.subscribe(subscriptions);
  t.add_cleanup(() => registration.cookies.unsubscribe(subscriptions));

  let cookie_change_promise = RunOnceCookieChangeReceivedPromise();

  await cookieStore.set({
    name: 'cookie-name',
    value: 'value',
    partitioned: true,
  });
  t.add_cleanup(async () => {
    await cookieStore.delete({
      name: 'cookie-name',
      partitioned: true,
    });
  });

  // Observes original cookie.
  let event = await cookie_change_promise;
  assert_equals(event.type, 'cookiechange');
  assert_equals(event.changed.length, 1);
  assert_equals(event.changed[0].name, 'cookie-name');
  assert_equals(event.changed[0].value, 'value');
  assert_equals(event.deleted.length, 0);

  cookie_change_promise = RunOnceCookieChangeReceivedPromise();

  // Duplicate overwrite should not be observed.
  await cookieStore.set({
    name: 'cookie-name',
    value: 'value',
    partitioned: true,
  });

  // This cookie should instead.
  await cookieStore.set({
    name: 'alternate-cookie-name',
    value: 'ignore',
    partitioned: true,
  });
  t.add_cleanup(async () => {
    await cookieStore.delete({
      name: 'alternate-cookie-name',
      partitioned: true,
    });
  });

  event = await cookie_change_promise;
  assert_equals(event.type, 'cookiechange');
  assert_equals(event.changed.length, 1);
  assert_equals(event.changed[0].name, 'alternate-cookie-name');
  assert_equals(event.changed[0].value, 'ignore');
  assert_equals(event.deleted.length, 0);
});
