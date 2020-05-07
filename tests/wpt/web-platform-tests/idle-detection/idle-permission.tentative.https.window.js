// META: script=/resources/testdriver.js
// META: script=/resources/testdriver-vendor.js
'use strict';

promise_test(async t => {
    await test_driver.set_permission(
        { name: 'notifications' }, 'denied', false);

    let status = new IdleDetector();
    await promise_rejects_dom(t, 'NotAllowedError', status.start());
}, "Deny notifications permission should work.");

promise_test(async t => {
    await test_driver.set_permission(
        { name: 'notifications' }, 'granted', false);

    let status = new IdleDetector();
    await status.start();

    assert_true(['active', 'idle'].includes(status.state.user),
                  'status has a valid user state');
    assert_true(['locked', 'unlocked'].includes(status.state.screen),
                  'status has a valid screen state');
}, "Grant notifications permission should work.");
