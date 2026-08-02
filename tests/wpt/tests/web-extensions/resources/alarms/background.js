const alarmName = 'wpt_test_alarm';

browser.test.runTests([
  /**
   * Tests `browser.alarms.create` and `browser.alarms.get`.
   */
  async function testAlarmsCreateAndGet() {
    await browser.alarms.clearAll();
    const now = Date.now();
    await browser.alarms.create(alarmName,
                                {delayInMinutes: 10, periodInMinutes: 5});
    const alarm = await browser.alarms.get(alarmName);
    browser.test.assertEq(typeof alarm, 'object',
                          'Retrieved alarm should be an object');
    browser.test.assertEq(alarm.name, alarmName, 'Alarm name should match');
    browser.test.assertEq(alarm.periodInMinutes, 5,
                          'Alarm periodInMinutes should match');
    browser.test.assertTrue(
        alarm.scheduledTime >= now,
        'scheduledTime should be greater than or equal to creation time');
    await browser.alarms.clearAll();
  },

  /**
   * Tests `browser.alarms.getAll`.
   */
  async function testAlarmsGetAll() {
    await browser.alarms.clearAll();
    await browser.alarms.create(alarmName, {delayInMinutes: 10});
    const alarms = await browser.alarms.getAll();
    browser.test.assertTrue(Array.isArray(alarms),
                            'getAll should return an array');
    const found = alarms.some((a) => a.name === alarmName);
    browser.test.assertTrue(found, 'getAll should include the created alarm');
    await browser.alarms.clearAll();
  },

  /**
   * Tests `browser.alarms.clear`.
   */
  async function testAlarmsClear() {
    await browser.alarms.clearAll();
    await browser.alarms.create(alarmName, {delayInMinutes: 10});
    const cleared = await browser.alarms.clear(alarmName);
    browser.test.assertTrue(cleared,
                            'clear should return true for existing alarm');
    const alarm = await browser.alarms.get(alarmName);
    browser.test.assertEq(alarm, undefined,
                          'Alarm should be undefined after clear');
    await browser.alarms.clearAll();
  },

  /**
   * Tests `browser.alarms.clearAll`.
   */
  async function testAlarmsClearAll() {
    await browser.alarms.clearAll();
    await browser.alarms.create('alarm1', {delayInMinutes: 10});
    await browser.alarms.create('alarm2', {delayInMinutes: 10});
    const cleared = await browser.alarms.clearAll();
    browser.test.assertTrue(cleared, 'clearAll should return true');
    const alarms = await browser.alarms.getAll();
    browser.test.assertEq(alarms.length, 0,
                          'Alarms array should be empty after clearAll');
  },

  /**
   * Tests `browser.alarms.onAlarm` event firing.
   */
  async function testAlarmsOnAlarm() {
    await browser.alarms.clearAll();
    const fireAlarmName = 'wpt_fire_alarm';
    const alarmPromise = new Promise((resolve) => {
      const listener = (alarm) => {
        if (alarm.name === fireAlarmName) {
          browser.alarms.onAlarm.removeListener(listener);
          resolve(alarm);
        }
      };
      browser.alarms.onAlarm.addListener(listener);
    });

    // Create alarm scheduled to fire in 1 second.
    await browser.alarms.create(fireAlarmName, {when: Date.now() + 1000});

    const firedAlarm = await alarmPromise;
    browser.test.assertEq(firedAlarm.name, fireAlarmName,
                          'Fired alarm name should match');
    await browser.alarms.clearAll();
  },

  /**
   * Tests `browser.alarms.create` for various name combinations.
   */
  async function testAlarmsCreateNames() {
    // The alarm name can be passed as a parameter.
    await browser.alarms.clearAll();
    await browser.alarms.create(alarmName,
                                {delayInMinutes: 10, periodInMinutes: 5});
    const alarm1 = await browser.alarms.get(alarmName);
    browser.test.assertEq(alarm1.name, alarmName, 'Alarm name should match');

    // The alarm name can be passed within the object.
    await browser.alarms.clearAll();
    await browser.alarms.create(
        {name: alarmName, delayInMinutes: 10, periodInMinutes: 5});
    const alarm2 = await browser.alarms.get(alarmName);
    browser.test.assertEq(alarm2.name, alarmName, 'Alarm name should match');

    // The alarm name can be omitted.
    await browser.alarms.clearAll();
    await browser.alarms.create({delayInMinutes: 10, periodInMinutes: 5});
    const alarm3 = await browser.alarms.get();
    browser.test.assertEq('', alarm3.name, 'Alarm name should be empty');
  },

  /**
   * Tests that `browser.alarms.create()` rejects when the alarm name is
   * specified both as a positional string argument and in the `alarmInfo`
   * object parameter. We expect the returned promise to reject with an error.
   */
  function testAlarmsCreateNameBothWaysError() {
    // We manually implement this validation since
    // `browser.test.assertThrows()` evaluates synchronously and cannot handle
    // asynchronous methods.
    return browser.alarms
        .create(alarmName, {name: alarmName, delayInMinutes: 10})
        .then(() => Promise.reject(
                  new Error('create should reject when name is passed twice')),
              () => {} /* Expected rejection. */);
  },

  /**
   * Tests synchronous `browser.alarms` error cases for various methods.
   * We expect these calls with invalid argument types to synchronously throw
   * errors.
   */
  function testAlarmsErrorCases() {
    // `create` throws when passed invalid alarmInfo (not an object).
    browser.test.assertThrows(() =>
                                  browser.alarms.create('invalid', 'invalid'));

    // `get` throws when passed invalid name (not a string).
    browser.test.assertThrows(() => browser.alarms.get(123));

    // `clear` throws when passed invalid name (not a string).
    browser.test.assertThrows(() => browser.alarms.clear(123));
  }
]);
