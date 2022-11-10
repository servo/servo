'use strict';

test(t => {
  const observer = new PressureObserver(
      t.unreached_func('oops should not end up here'), {sampleRate: 1.0});
  t.add_cleanup(() => observer.disconnect());
  assert_throws_js(TypeError, () => {
    observer.observe('random');
  });
}, 'PressureObserver.observe() requires a valid source');

test(t => {
  const observer = new PressureObserver(
      t.unreached_func('oops should not end up here'), {sampleRate: 1.0});
  t.add_cleanup(() => observer.disconnect());
  assert_throws_js(TypeError, () => {
    observer.unobserve('random');
  });
}, 'PressureObserver.unobserve() requires a valid source');
