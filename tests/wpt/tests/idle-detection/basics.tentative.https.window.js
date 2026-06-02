// META: script=/resources/testdriver.js
// META: script=/resources/testdriver-vendor.js
// META: title=Idle Detection API: Basics

'use strict';

promise_setup(async t => {
  await test_driver.set_permission({name: 'idle-detection'}, 'granted');
})

promise_test(async t => {
  let detector = new IdleDetector();
  let watcher = new EventWatcher(t, detector, ["change"]);
  let initial_state = watcher.wait_for("change");

  await detector.start();
  await initial_state;

  assert_true(['active', 'idle'].includes(detector.userState),
                'has a valid user state');
  assert_true(['locked', 'unlocked'].includes(detector.screenState),
                'has a valid screen state');
}, 'start() basics');

promise_test(async t => {
  let used = false;

  const detector = new IdleDetector();
  detector.start({
    get threshold() {
      used = true;
      return 60000;
    }
  });

  assert_true(used, 'start() options "threshold" member was used');
}, 'start() uses threshold property');

promise_test(async t => {
  let used = false;

  const controller = new AbortController();
  const detector = new IdleDetector();
  detector.start({
    get signal() {
      used = true;
      return controller.signal;
    }
  });

  assert_true(used, 'start() options "signal" member was used');
}, 'start() uses signal property');


promise_test(async t => {
  const detector = new IdleDetector();
  await promise_rejects_js(t, TypeError, detector.start({threshold: 0}));
}, 'start() rejects with invalid threshold (0)');

promise_test(async t => {
  const detector = new IdleDetector();
  await promise_rejects_js(t, TypeError, detector.start({threshold: 59000}));
}, 'start() rejects with threshold below minimum (59000)');

promise_test(async t => {
  const detector = new IdleDetector();
  await detector.start({threshold: 60000});
}, 'start() rejects threshold (60000)');

promise_test(async t => {
  const detector = new IdleDetector();
  await detector.start({threshold: 61000});
}, 'start() allows threshold (61000)');

promise_test(async t => {
  const detector = new IdleDetector();
  await promise_rejects_js(t, TypeError, detector.start({threshold: null}));
}, 'start() rejects with invalid threshold (null)');

promise_test(async t => {
  const detector = new IdleDetector();
  await promise_rejects_js(t, TypeError, detector.start({threshold: -1}));
}, 'start() rejects with invalid threshold (-1)');

promise_test(async t => {
  const detector = new IdleDetector();
  await promise_rejects_js(t, TypeError, detector.start({threshold: NaN}));
}, 'start() rejects with invalid threshold (NaN)');

promise_test(async t => {
  const detector = new IdleDetector();
  await detector.start();
}, 'start() uses a default value for the threshold when none is passed');

promise_test(async t => {
  const detector = new IdleDetector();
  await detector.start({threshold: undefined});
}, 'start() uses a default value for the threshold');
