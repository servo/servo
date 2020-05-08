// META: script=/resources/testdriver.js
// META: script=/resources/testdriver-vendor.js
// META: title=Idle Detection API: Basics

'use strict';

promise_setup(async t => {
  await test_driver.set_permission({ name: 'notifications' }, 'granted', false);
})

promise_test(async t => {
  let status = new IdleDetector();
  let watcher = new EventWatcher(t, status, ["change"]);
  let initial_state = watcher.wait_for("change");

  await status.start();
  await initial_state;

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
      return 60000;
    }
  });

  assert_true(used, 'constructor options "threshold" member was used');
}, 'constructor uses threshold property');

promise_test(async t => {
  assert_throws_js(TypeError, () => new IdleDetector({threshold: 0}));
}, 'constructor throws with invalid threshold (0)');

promise_test(async t => {
  assert_throws_js(TypeError, () => new IdleDetector({threshold: 59000}));
}, 'constructor throws with threshold below minimum (59000)');

promise_test(async t => {
  new IdleDetector({threshold: 60000});
}, 'constructor allows threshold (60000)');

promise_test(async t => {
  new IdleDetector({threshold: 61000});
}, 'constructor allows threshold (61000)');

promise_test(async t => {
  assert_throws_js(TypeError, () => new IdleDetector({threshold: null}));
}, 'constructor throws with invalid threshold (null)');

promise_test(async t => {
  assert_throws_js(TypeError, () => new IdleDetector({threshold: -1}));
}, 'constructor throws with invalid threshold (-1)');

promise_test(async t => {
  assert_throws_js(TypeError, () => new IdleDetector({threshold: NaN}));
}, 'constructor throws with invalid threshold (NaN)');

promise_test(async t => {
  new IdleDetector();
}, 'constructor uses a default value for the threshold when none is passed');

promise_test(async t => {
  new IdleDetector({threshold: undefined});
}, 'constructor uses a default value for the threshold');
