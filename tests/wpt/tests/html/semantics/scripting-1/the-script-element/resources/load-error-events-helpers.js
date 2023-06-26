"use strict";
// Helper functions to be used from load-error-events*.html tests.

function event_test(name, load_to_be_fired, error_to_be_fired) {
  return {
      test: async_test(name),
      executed: false,
      load_event_to_be_fired: load_to_be_fired,
      error_event_to_be_fired: error_to_be_fired
  };
}

// Should be used as load/error event handlers of script tags,
// with |t| = the object returned by event_test().
function onLoad(t) {
    t.test.step(function() {
        if (t.load_event_to_be_fired) {
            assert_true(t.executed,
                'Load event should be fired after script execution');
            // Delay done() a little so that if an error event happens
            // the assert_unreached is reached and fails the test.
            t.test.step_timeout(() => t.test.done(), 100);
        } else {
            assert_unreached('Load event should not be fired.');
        }
    });
};
function onError(t) {
    t.test.step(function() {
        if (t.error_event_to_be_fired) {
            assert_false(t.executed);
            // Delay done() a little so that if a load event happens
            // the assert_unreached is reached and fails the test.
            t.test.step_timeout(() => t.test.done(), 100);
        } else {
            assert_unreached('Error event should not be fired.');
        }
    });
};

// To be called from inline scripts, which expect no load/error events.
function onExecute(t) {
    t.executed = true;
    // Delay done() a little so that if a load/error event happens
    // the assert_unreached is reached and fails the test.
    t.test.step_timeout(() => t.test.done(), 100);
}
