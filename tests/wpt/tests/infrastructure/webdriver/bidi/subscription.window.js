// META: title=Can subscribe and receive WebDriver BiDi events
// META: script=/resources/testdriver.js?feature=bidi

'use strict';

promise_test(async () => {
    const some_message = "SOME MESSAGE";
    // Subscribe to `log.entryAdded` BiDi events. This will not add a listener to the page.
    await test_driver.bidi.log.entry_added.subscribe();
    // Add a listener for the log.entryAdded event. This will not subscribe to the event, so the subscription is
    // required before. The cleanup is done automatically after the test is finished.
    const log_entry_promise = test_driver.bidi.log.entry_added.once();
    // Emit a console.log message.
    // Note: Lint rule is disabled in `lint.ignore` file.
    console.log(some_message);
    // Wait for the log.entryAdded event to be received.
    const event = await log_entry_promise;
    // Assert the log.entryAdded event has the expected message.
    assert_equals(event.args.length, 1);
    const event_message = event.args[0];
    assert_equals(event_message.value, some_message);
}, "Assert testdriver can subscribe and receive events");
