// META: script=resources/utils.js

async_test(t => {
  assert_implements(window.PerformanceLongTaskTiming, 'Longtasks are not supported.');
  // Create a long task before any observer.
  const begin = window.performance.now();
  while (window.performance.now() < begin + 60);
  // After a timeout, add an observer with buffered flag.
  t.step_timeout(() => {
    new PerformanceObserver(t.step_func_done(list => {
      list.getEntries().forEach(entry => {
        checkLongTaskEntry(entry);
      });
    })).observe({type: 'longtask', buffered: true});
  }, 0);
}, 'PerformanceObserver with buffered flag can see previous longtask entries.');
