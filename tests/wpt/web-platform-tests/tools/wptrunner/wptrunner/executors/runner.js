document.title = '%(title)s';

window.addEventListener(
  "message",
  function(event) {
    window.message_queue.push(event);
    window.process_next_event();
  },
  false
);


window.process_next_event = function() {
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
  if (!window.testdriver_callback) {
    return;
  }
  var event = window.message_queue.shift();
  if (!event) {
    return;
  }
  var data = event.data;

  var payload = undefined;

  switch(data.type) {
  case "complete":
    var tests = event.data.tests;
    var status = event.data.status;

    var subtest_results = tests.map(function(x) {
      return [x.name, x.status, x.message, x.stack];
    });
    payload = [status.status,
               status.message,
               status.stack,
               subtest_results];
    clearTimeout(window.timer);
    break;
  case "action":
    payload = data;
    break;
  default:
    return;
  }
  var callback = window.testdriver_callback;
  window.testdriver_callback = null;
  callback([window.url, data.type, payload]);
};
