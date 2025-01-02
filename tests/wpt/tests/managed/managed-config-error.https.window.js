promise_test(t => {
  return promise_rejects_js(
      t, TypeError, navigator.managed.getManagedConfiguration(-1));
}, 'Number instead of keys');

promise_test(t => {
  return promise_rejects_js(
      t, TypeError, navigator.managed.getManagedConfiguration());
}, 'Empty key list');

promise_test(t => {
  return promise_rejects_js(
      t, TypeError, navigator.managed.getManagedConfiguration({'a': 2}));
}, 'Dictionary instead of list');
