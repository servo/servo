// META: script=/resources/testdriver.js
// META: script=/resources/testdriver-vendor.js
'use strict';

promise_test(async t => {
  await test_driver.set_permission({name: 'idle-detection'}, 'denied', false);

  let detector = new IdleDetector();
  await promise_rejects_dom(t, 'NotAllowedError', detector.start());
}, 'Denying idle-detection permission should block access.');

promise_test(async t => {
  await test_driver.set_permission({name: 'idle-detection'}, 'granted', false);

  let detector = new IdleDetector();
  await detector.start();

  assert_true(
      ['active', 'idle'].includes(detector.userState),
      'has a valid user state');
  assert_true(
      ['locked', 'unlocked'].includes(detector.screenState),
      'has a valid screen state');
}, 'Granting idle-detection permission should allow access.');

promise_test(async t => {
  await test_driver.set_permission({name: 'idle-detection'}, 'prompt', false);

  await promise_rejects_dom(t, 'NotAllowedError', IdleDetector.requestPermission());

  await test_driver.bless('request permission');
  let state = await IdleDetector.requestPermission();
  assert_equals(state, 'prompt');
}, 'The idle-detection permission cannot be requested without a user gesture');
