class MessageQueue {
  constructor() {
    this._queue = [];
  }

  push(item) {
    this._queue.push(item);
    __wptrunner_process_next_event();
  }

  shift() {
    return this._queue.shift();
  }
}

window.__wptrunner_testdriver_callback = null;
window.__wptrunner_message_queue = new MessageQueue();
window.__wptrunner_url = null;

window.__wptrunner_process_next_event = function() {
  /* This function handles the next testdriver event. The presence of
     window.testdriver_callback is used as a switch; when that function
     is present we are able to handle the next event and when is is not
     present we must wait. Therefore to drive the event processing, this
     function must be called in two circumstances:
       * Every time there is a new event that we may be able to handle
       * Every time we set the callback function
     This function unsets the callback, so no further testdriver actions
     will be run until it is reset, which wptrunner does after it has
     completed handling the current action.
   */

  if (!window.__wptrunner_testdriver_callback) {
    return;
  }
  var data = window.__wptrunner_message_queue.shift();
  if (!data) {
    return;
  }

  var payload = undefined;

  switch(data.type) {
  case "complete":
    var tests = data.tests;
    var status = data.status;

    var subtest_results = tests.map(function(x) {
      return [x.name, x.status, x.message, x.stack];
    });
    payload = [status.status,
               status.message,
               status.stack,
               subtest_results];
    clearTimeout(window.__wptrunner_timer);
    break;
  case "action":
    payload = data;
    break;
  default:
    return;
  }
  var callback = window.__wptrunner_testdriver_callback;
  window.__wptrunner_testdriver_callback = null;
  callback([__wptrunner_url, data.type, payload]);
};

(function() {
  var props = {output: %(output)d,
               timeout_multiplier: %(timeout_multiplier)s,
               explicit_timeout: %(explicit_timeout)s,
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

