async_test(function(t) {
  var counter = 0;
  onmessage = function() {
    counter++;
  }
  function perf_observer(list, observer) {
    // The timeline event should be throttled,
    // while the event-loop is busy,
    // and only handled after all img error related events,
    // across two iterations of the event-loop.
    assert_equals(counter, 20)
    t.done();
  }
  var observer2 = new PerformanceObserver(perf_observer);
  observer2.observe({entryTypes: ["mark"]});

  for (var i = 0; i < 10; i++) {
    postMessage(new Blob("test"), "*");
  }

  // Do this in the current iteration of the event-loop.
  performance.mark("start");

  setTimeout(function() {
    for (var i = 0; i < 10; i++) {
      postMessage(new Blob("test"), "*");
    }
  }, 0)
});
