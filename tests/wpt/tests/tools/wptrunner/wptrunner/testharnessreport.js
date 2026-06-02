(function() {
  // Signal to `testdriver.js` that this is the "main" test browsing context,
  // meaning testdriver actions should be queued for retrieval instead of
  // `postMessage()`d elsewhere.
  window.__wptrunner_is_test_context = true;

  var props = {output: %(output)d,
               timeout_multiplier: %(timeout_multiplier)s,
               explicit_timeout: %(explicit_timeout)s,
               debug: %(debug)s,
               message_events: ["completion"]};

  add_completion_callback(function(tests, harness_status) {
    __wptrunner_message_queue.push({
      "type": "complete",
      "tests": tests,
      "status": harness_status});
    __wptrunner_process_next_event();
  });
  setup(props);
})();
