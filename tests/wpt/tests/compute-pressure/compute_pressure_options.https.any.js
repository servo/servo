// META: global=window,dedicatedworker,sharedworker

'use strict';

test(t => {
  const observer = new PressureObserver(() => {}, {sampleInterval: 0});
  assert_equals(typeof observer, 'object');
}, 'PressureObserver constructor doesnt throw error for sampleInterval value 0');


test(t => {
  assert_throws_js(TypeError, () => {
    new PressureObserver(() => {}, {sampleInterval: -2});
  });
}, 'PressureObserver constructor requires a positive sampleInterval');

test(t => {
  assert_throws_js(TypeError, () => {
    new PressureObserver(() => {}, {sampleInterval: 2 ** 32});
  });
}, 'PressureObserver constructor requires a sampleInterval in unsigned long range');

test(t => {
  const observer = new PressureObserver(() => {}, {});
  assert_equals(typeof observer, 'object');
}, 'PressureObserver constructor succeeds on empty sampleInterval');
