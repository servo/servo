// META: title=Cookie Store API: cookiechange event in ServiceWorker with already-expired cookie.
// META: global=serviceworker

'use strict';

const kScope = '/cookie-store/does/not/exist';

function WorkerActivationPromise() {
  return new Promise((resolve) => {
    if (registration.active) {
      resolve();
      return;
    }
    self.addEventListener('activate', () => { resolve(); });
  });
}

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

  const eventPromise = RunOnceCookieChangeReceivedPromise();

  await cookieStore.set({
    name: 'cookie-name',
    value: 'already-expired',
    expires: new Date(new Date() - 10_000),
  });

  await cookieStore.set('another-cookie-name', 'ignore');
  t.add_cleanup(() => cookieStore.delete('another-cookie-name'));

  const event = await eventPromise;
  assert_equals(event.type, 'cookiechange');
  assert_equals(event.changed.length, 1);
  assert_equals(event.changed[0].name, 'another-cookie-name');
  assert_equals(event.changed[0].value, 'ignore');
  assert_equals(event.deleted.length, 0);
});

promise_test(async t => {
  await WorkerActivationPromise();

  const subscriptions = [{url: `${kScope}/path`}];
  await registration.cookies.subscribe(subscriptions);
  t.add_cleanup(() => registration.cookies.unsubscribe(subscriptions));

  const eventPromise = RunOnceCookieChangeReceivedPromise();

  await cookieStore.set({
    name: 'cookie-name',
    value: 'already-expired',
    expires: new Date(new Date() - 10_000),
    partitioned: true,
  });

  await cookieStore.set({
    name: 'another-cookie-name',
    value: 'ignore',
    partitioned: true,
  });
  t.add_cleanup(() => cookieStore.delete({
    name: 'another-cookie-name',
    partitioned: true,
  }));

  const event = await eventPromise;
  assert_equals(event.type, 'cookiechange');
  assert_equals(event.changed.length, 1);
  assert_equals(event.changed[0].name, 'another-cookie-name');
  assert_equals(event.changed[0].value, 'ignore');
  assert_equals(event.deleted.length, 0);
});
