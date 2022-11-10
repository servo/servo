// META: script=/resources/test-only-api.js
// META: script=resources/pressure-helpers.js

'use strict';

pressure_test(async (t, mockPressureService) => {
  const pressureChanges = await new Promise(async resolve => {
    const observer_changes = [];
    let n = 0;
    const observer = new PressureObserver(changes => {
      observer_changes.push(changes);
      if (++n === 2)
        resolve(observer_changes);
    }, {sampleRate: 1.0});
    observer.observe('cpu');
    const updatesDelivered = mockPressureService.updatesDelivered();
    mockPressureService.setPressureUpdate('critical');
    mockPressureService.startPlatformCollector(/*sampleRate*/ 1.0);
    // Deliver 2 updates.
    await t.step_wait(
        () => mockPressureService.updatesDelivered() >= (updatesDelivered + 2),
        'Wait for more than one update to be delivered to the observer');
    mockPressureService.setPressureUpdate('nominal');
    // Deliver more updates, |resolve()| will be called when the new pressure
    // state reaches PressureObserver and its callback is invoked
    // for the second time.
  });
  assert_equals(pressureChanges.length, 2);
  assert_equals(pressureChanges[0][0].state, 'critical');
  assert_equals(pressureChanges[1][0].state, 'nominal');
}, 'Changes that fail the "has change in data" test are discarded.');

pressure_test(async (t, mockPressureService) => {
  const pressureChanges = await new Promise(async resolve => {
    const observer_changes = [];
    let n = 0;
    const observer = new PressureObserver(changes => {
      observer_changes.push(changes);
      if (++n === 2)
        resolve(observer_changes);
    }, {sampleRate: 1.0});
    observer.observe('cpu');
    const updatesDelivered = mockPressureService.updatesDelivered();
    mockPressureService.setPressureUpdate('critical', ['thermal']);
    mockPressureService.startPlatformCollector(/*sampleRate*/ 1.0);

    // Deliver 2 updates.
    await t.step_wait(
        () => mockPressureService.updatesDelivered() >= (updatesDelivered + 2),
        'Wait for more than one update to be delivered to the observer');
    mockPressureService.setPressureUpdate('critical', ['power-supply']);
    // Deliver more updates, |resolve()| will be called when the new pressure
    // state reaches PressureObserver and its callback is invoked
    // for the second time.
  });
  assert_equals(pressureChanges.length, 2);
  assert_equals(pressureChanges[0][0].state, 'critical');
  assert_equals(pressureChanges[0][0].factors[0], 'thermal');
  assert_equals(pressureChanges[1][0].state, 'critical');
  assert_equals(pressureChanges[1][0].factors[0], 'power-supply');
}, 'Factors that fail the "has change in data" test are discarded.');
