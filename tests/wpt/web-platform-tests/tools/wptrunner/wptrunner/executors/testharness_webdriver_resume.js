var callback = arguments[arguments.length - 1];

function process_event(event) {
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
    window.setMessageListener(function(event) {
      window.message_queue.push(event);
    });
    payload = data;
    break;
  default:
    return;
  }

  callback(["%(url)s", data.type, payload]);
}

window.removeEventListener("message", window.current_listener);
if (window.message_queue.length) {
  var next = window.message_queue.shift();
  process_event(next);
} else {
  window.addEventListener(
    "message", function f(event) {
      window.removeEventListener("message", f);
      process_event(event);
    }, false);
}
