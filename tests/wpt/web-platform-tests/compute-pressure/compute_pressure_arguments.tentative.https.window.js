'use strict';

for (const property of ['cpuUtilizationThresholds', 'cpuSpeedThresholds']) {
  for (const out_of_range_value of [-1.0, 0.0, 1.0, 2.0]) {
    test(t => {
      const callback = () => {};

      const options = {
          cpuUtilizationThresholds: [0.5], cpuSpeedThresholds: [0.5] };
      options[property] = [out_of_range_value];

      assert_throws_js(TypeError, () => {
        new ComputePressureObserver(callback, options);
      });
    }, `ComputePressureObserver constructor throws when ${property} ` +
       `is [${out_of_range_value}]`);
  }

  for (const valid_value of [0.05, 0.1, 0.2, 0.5, 0.9, 0.95]) {
    test(t => {
      const callback = () => {};

      const options = {
          cpuUtilizationThresholds: [0.5], cpuSpeedThresholds: [0.5] };
      options[property] = [valid_value];

      const observer = new ComputePressureObserver(callback, options);
      assert_true(observer instanceof ComputePressureObserver);
    }, `ComputePressureObserver constructor accepts ${property} value ` +
       `[${valid_value}]`);
  }

  promise_test(async t => {
    const many_thresholds = [0.5];
    for (let i = 0.01; i < 0.5; i += 0.0001) {
      many_thresholds.push(0.5 + i);
      many_thresholds.push(0.5 - i);
    }

    const options = {
        cpuUtilizationThresholds: [0.5], cpuSpeedThresholds: [0.5] };
    options[property] = many_thresholds;

    const update = await new Promise((resolve, reject) => {
      const observer = new ComputePressureObserver(resolve, options);
      t.add_cleanup(() => observer.stop());
      observer.observe().catch(reject);
    });

    const effective_thresholds = update.options[property];
    assert_less_than(effective_thresholds.length, many_thresholds.length,
                     'only a small number of thresholds are selected');

    const expected_thresholds =
        many_thresholds.slice(0, effective_thresholds.length);
    expected_thresholds.sort();
    assert_array_equals(
        effective_thresholds, expected_thresholds,
        'thresholds are selected in the given order, before sorting');
  }, `ComputePressureObserver filters thresholds in ${property}`);
}

test(t => {
  const callback = () => {};


  assert_throws_js(TypeError, () => {
    new ComputePressureObserver(
        callback,
        { cpuUtilizationThresholds: [0.5, 0.5], cpuSpeedThresholds: [0.5] });
  });
}, 'ComputePressureObserver constructor throws when cpuUtilizationThresholds ' +
   'has duplicates');
