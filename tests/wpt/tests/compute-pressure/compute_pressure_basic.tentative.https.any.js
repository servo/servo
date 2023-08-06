// META: timeout=long
// META: script=/resources/test-only-api.js
// META: script=resources/pressure-helpers.js
// META: global=window,dedicatedworker,sharedworker

'use strict';

pressure_test((t, mockPressureService) => {
  const observer = new PressureObserver(() => {
    assert_unreached('The observer callback should not be called');
  });

  mockPressureService.setExpectedFailure(
      new DOMException('', 'NotSupportedError'));
  return promise_rejects_dom(t, 'NotSupportedError', observer.observe('cpu'));
}, 'Return NotSupportedError when calling observer()');

pressure_test(async (t, mockPressureService) => {
  const changes = await new Promise(resolve => {
    const observer = new PressureObserver(resolve);
    t.add_cleanup(() => observer.disconnect());
    observer.observe('cpu');
    mockPressureService.setPressureUpdate('cpu', 'critical');
    mockPressureService.startPlatformCollector(/*sampleRate=*/ 5.0);
  });
  assert_true(changes.length === 1);
  assert_equals(changes[0].state, 'critical');
  assert_equals(changes[0].source, 'cpu');
  assert_equals(typeof changes[0].time, 'number');
}, 'Basic functionality test');

pressure_test((t, mockPressureService) => {
  const observer = new PressureObserver(() => {
    assert_unreached('The observer callback should not be called');
  });

  const promise = observer.observe('cpu');
  observer.unobserve('cpu');
  mockPressureService.setPressureUpdate('cpu', 'critical');
  mockPressureService.startPlatformCollector(/*sampleRate=*/ 5.0);

  return promise_rejects_dom(t, 'AbortError', promise);
}, 'Removing observer before observe() resolves works');

pressure_test(async (t, mockPressureService) => {
  const callbackPromises = [];
  const observePromises = [];

  for (let i = 0; i < 2; i++) {
    callbackPromises.push(new Promise(resolve => {
      const observer = new PressureObserver(resolve);
      t.add_cleanup(() => observer.disconnect());
      observePromises.push(observer.observe('cpu'));
    }));
  }

  await Promise.all(observePromises);

  mockPressureService.setPressureUpdate('cpu', 'critical');
  mockPressureService.startPlatformCollector(/*sampleRate=*/ 5.0);

  return Promise.all(callbackPromises);
}, 'Calling observe() multiple times works');

pressure_test(async (t, mockPressureService) => {
  const observer1_changes = [];
  await new Promise(resolve => {
    const observer1 = new PressureObserver(changes => {
      observer1_changes.push(changes);
      resolve();
    });
    t.add_cleanup(() => observer1.disconnect());
    observer1.observe('cpu');
    mockPressureService.setPressureUpdate('cpu', 'critical');
    mockPressureService.startPlatformCollector(/*sampleRate=*/ 5.0);
  });
  assert_true(observer1_changes.length === 1);
  assert_equals(observer1_changes[0][0].source, 'cpu');
  assert_equals(observer1_changes[0][0].state, 'critical');

  const observer2_changes = [];
  await new Promise(resolve => {
    const observer2 = new PressureObserver(changes => {
      observer2_changes.push(changes);
      resolve();
    });
    t.add_cleanup(() => observer2.disconnect());
    observer2.observe('cpu');
  });
  assert_true(observer2_changes.length === 1);
  assert_equals(observer2_changes[0][0].source, 'cpu');
  assert_equals(observer2_changes[0][0].state, 'critical');
}, 'Starting a new observer after an observer has started works');
