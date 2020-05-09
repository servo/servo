// META: script=/resources/testdriver.js
// META: script=/resources/testdriver-vendor.js
'use strict';

promise_test(async t => {
    await test_driver.set_permission(
        { name: 'notifications' }, 'denied', false);

    let detector = new IdleDetector();
    await promise_rejects_dom(t, 'NotAllowedError', detector.start());
}, "Deny notifications permission should work.");

promise_test(async t => {
    await test_driver.set_permission(
        { name: 'notifications' }, 'granted', false);

    let detector = new IdleDetector();
    await detector.start();

    assert_true(['active', 'idle'].includes(detector.userState),
                  'has a valid user state');
    assert_true(['locked', 'unlocked'].includes(detector.screenState),
                  'has a valid screen state');
}, "Grant notifications permission should work.");
