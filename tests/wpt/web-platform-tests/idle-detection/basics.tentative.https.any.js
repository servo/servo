// META: title=Idle Detection API: Basics

'use strict';

promise_test(async t => {
  let status = new IdleDetector();

  let watcher = new EventWatcher(t, status, ["change"]);

  await status.start();

  await watcher.wait_for("change");

  assert_true(['active', 'idle'].includes(status.state.user),
                'status has a valid user state');
  assert_true(['locked', 'unlocked'].includes(status.state.screen),
                'status has a valid screen state');

}, 'start() basics');

promise_test(async t => {
  let used = false;

  new IdleDetector({
    get threshold() {
      used = true;
      return 60;
    }
  });

  assert_true(used, 'constructor options "threshold" member was used');
}, 'constructor uses threshold property');

promise_test(async t => {
  try {
    new IdleDetector({threshold: 0});
    assert_unreached('Threshold under 60 should reject');
  } catch (error) {
    assert_equals(error.name, 'TypeError');
  }
}, 'constructor throws with invalid threshold (0)');

promise_test(async t => {
  try {
    new IdleDetector({threshold: 59});
    assert_unreached('Threshold under 60 should reject');
  } catch (error) {
    assert_equals(error.name, 'TypeError');
  }
}, 'constructor throws with threshold below minimum (59)');

promise_test(async t => {
  new IdleDetector({threshold: 60});
}, 'constructor allows threshold (60)');

promise_test(async t => {
  new IdleDetector({threshold: 61});
}, 'constructor allows threshold (61)');

promise_test(async t => {
  try {
    new IdleDetector({threshold: null});
    assert_unreached('Threshold of null should reject');
  } catch (error) {
    assert_equals(error.name, 'TypeError');
  }
}, 'constructor throws with invalid threshold (null)');

promise_test(async t => {
  try {
    new IdleDetector({threshold: -1});
    assert_unreached('Threshold of negative numbers should reject');
  } catch (error) {
    assert_equals(error.name, 'TypeError');
  }
}, 'constructor throws with invalid threshold (-1)');

promise_test(async t => {
  try {
    new IdleDetector({threshold: NaN});
    assert_unreached('Threshold of NaN should reject');
  } catch (error) {
    assert_equals(error.name, 'TypeError');
  }
}, 'constructor throws with invalid threshold (NaN)');

promise_test(async t => {
  new IdleDetector();
}, 'constructor uses a default value for the threshold when none is passed');

promise_test(async t => {
  new IdleDetector({threshold: undefined});
}, 'constructor uses a default value for the threshold');

