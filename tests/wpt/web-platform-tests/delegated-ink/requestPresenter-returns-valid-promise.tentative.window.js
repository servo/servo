test(() => {
  assert_implements(typeof navigator.ink !== "undefined", 'ink is not supported');
}, "navigator needs to support ink to run this test.");

promise_test(t => {
  return promise_rejects_js(t, TypeError, navigator.ink.requestPresenter('bad-type'));
}, "Receive rejected promise for a bad type.");

promise_test(() => {
  return navigator.ink.requestPresenter('delegated-ink-trail');
}, "Received fulfilled promise for a good type.");