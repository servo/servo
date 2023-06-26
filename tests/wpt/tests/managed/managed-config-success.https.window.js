// META: script=/resources/test-only-api.js
// META: script=resources/managed-configuration-helper.js

'use strict'

managed_config_test(async (test, managedConfigTest) => {
  promise_rejects_dom(
      test, 'NotAllowedError',
      navigator.managed.getManagedConfiguration(['a']));
}, 'App is not managed.');

managed_config_test(async (test, managedConfigTest) => {
  managedConfigTest.setManagedConfig({a: 2});
  const result = await navigator.managed.getManagedConfiguration(['a']);
  assert_equals(Object.keys(result).length, 1);
  assert_equals(result.a, 2);
}, 'Configuration is returned');

managed_config_test(async (test, managedConfigTest) => {
  managedConfigTest.setManagedConfig({a: 2, b: 3, c: 1});
  const result = await navigator.managed.getManagedConfiguration(['b', 'c']);
  assert_equals(Object.keys(result).length, 2);
  assert_equals(result.b, 3);
  assert_equals(result.c, 1);
}, 'Selected keys are returned.');

managed_config_test(async (test, managedConfigTest) => {
  managedConfigTest.setManagedConfig({a: 2, b: 3, c: 1});
  const result = await navigator.managed.getManagedConfiguration(['b', 'e']);
  assert_equals(Object.keys(result).length, 1);
  assert_equals(result.b, 3);
}, 'Only existing keys are returned.');

managed_config_test(async (test, managedConfigTest) => {
  managedConfigTest.setManagedConfig({a: 2, b: false, c: {x: 3}});
  const result =
      await navigator.managed.getManagedConfiguration(['a', 'b', 'c']);
  assert_equals(Object.keys(result).length, 3);
  assert_equals(result.a, 2);
  assert_equals(result.b, false);
  assert_equals(Object.keys(result.c).length, 1);
  assert_equals(result.c.x, 3);
}, 'Values can be of any type');

managed_config_test(async (test, managedConfigTest) => {
  const nextObserverAdded = managedConfigTest.nextObserverAdded();
  const watcher =
      new EventWatcher(test, navigator.managed, ['managedconfigurationchange']);
  await nextObserverAdded;
  const event = watcher.wait_for(['managedconfigurationchange']);
  managedConfigTest.setManagedConfig({'a': 2});
  await event;
  const result = await navigator.managed.getManagedConfiguration(['a']);
  assert_equals(Object.keys(result).length, 1);
  assert_equals(result.a, 2);
}, 'A change in managed configuration is observed.');
