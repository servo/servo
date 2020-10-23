async_test(t => {
  assert_implements(window.PerformanceLongTaskTiming, 'Longtasks are not supported.');
  // Create a long task before any observer.
  const begin = window.performance.now();
  while (window.performance.now() < begin + 60);
  // After a timeout, add an observer with buffered flag.
  t.step_timeout(() => {
    new PerformanceObserver(t.step_func_done(list => {
      list.getEntries().forEach(entry => {
        assert_equals(entry.entryType, 'longtask');
        assert_equals(entry.name, 'self');
        assert_greater_than(entry.duration, 50);
      });
    })).observe({type: 'longtask', buffered: true});
  }, 0);
}, 'PerformanceObserver with buffered flag can see previous longtask entries.');
