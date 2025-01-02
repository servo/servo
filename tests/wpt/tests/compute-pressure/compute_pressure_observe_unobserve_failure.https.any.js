// META: global=window,dedicatedworker,sharedworker

'use strict';

promise_test(async t => {
  const observer =
      new PressureObserver(t.unreached_func('oops should not end up here'));
  t.add_cleanup(() => observer.disconnect());
  await promise_rejects_js(t, TypeError, observer.observe('random'));
}, 'PressureObserver.observe() requires a valid source');

test(t => {
  const observer =
      new PressureObserver(t.unreached_func('oops should not end up here'));
  t.add_cleanup(() => observer.disconnect());
  assert_throws_js(TypeError, () => {
    observer.unobserve('random');
  });
}, 'PressureObserver.unobserve() requires a valid source');
