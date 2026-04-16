// META: title=Throttling the performance timeline task queue.

async_test(function(t) {
  var counter = 0;

  function perf_observer(list, observer) {
    // The timeline event should be throttled,
    // while the event-loop is busy,
    // and only handled after at least 6 other events,
    // across several iterations of the event-loop.
    assert_true(counter > 6)
  }
  var observer2 = new PerformanceObserver(t.step_func_done(perf_observer));
  observer2.observe({entryTypes: ["mark"]});

  for (var i = 0; i < 4; i++) {
      var reader = new FileReader();
      reader.onload = function() {
         counter++;
      };
      var blob = new Blob();
      reader.readAsText(blob);
  }

  var reader = new FileReader();
  reader.onload = function() {
     counter++;
     // In a subsequent iteration of the event-loop,
     // start reading another 5 blobs
     for (var i = 0; i < 5; i++) {
        var reader = new FileReader();
        reader.onload = function() {
           counter++;
        };
        var blob = new Blob();
        reader.readAsText(blob);
     }
  };
  var blob = new Blob();
  reader.readAsText(blob);
  // We've started reading 5 blobs in this iteration of the event-loop.

  // Do this in the current iteration of the event-loop.
  performance.mark("start");
});
