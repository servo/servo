window.customTimers = {};
// Create a custome timestamp with a custom name
function mark(name) {
  if (window.performance) {
    // performance.now() is the time after navigationStart
    // https://developer.mozilla.org/en-US/docs/Web/API/Performance/now
    var time = performance.now() + performance.timing.navigationStart;
  }
  else {
    var time = (new Date()).getTime();
  }
  window.customTimers[name] = time;
}

// Notifying the test harness that the test has ended, otherwise the test
// harness will time out
function done() {
  var elem = document.createElement('span')
  elem.id = "GECKO_TEST_DONE";
  document.body.appendChild(elem);
}

