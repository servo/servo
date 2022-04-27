'use strict';

promise_test(async t => {
  const observer = new ComputePressureObserver(
      t.unreached_func('oops should not end up here'),
      {cpuUtilizationThresholds: [0.1, 0.5], cpuSpeedThresholds: [0.5]});
  t.add_cleanup(() => observer.disconnect());
  await promise_rejects_js(t, TypeError, observer.observe('random'));
}, 'ComputePressureObserver.observe() requires a valid source');

test(function(t) {
  const observer = new ComputePressureObserver(
      t.unreached_func('oops should not end up here'),
      {cpuUtilizationThresholds: [0.1, 0.5], cpuSpeedThresholds: [0.5]});
  t.add_cleanup(() => observer.disconnect());
  assert_throws_js(TypeError, () => {
    observer.unobserve('random');
  });
}, 'ComputePressureObserver.unobserve() requires a valid source');
