async_test(t => {
  performance.mark('foo');
  // Use a timeout to ensure the remainder of the test runs after the entry is created.
  t.step_timeout(() => {
    // `buffered` flag set to true but with entryTypes so that
    // the `buffered` flag should be ignored, thus there should be no entry.
    new PerformanceObserver(() => {
      assert_unreached('Should not have observed any entry!');
    }).observe({entryTypes: ['mark'], buffered: true});
    // Use a timeout to give time to the observer.
    t.step_timeout(t.step_func_done(() => {}), 100);
  }, 0);
}, 'PerformanceObserver without buffered flag set to false cannot see past entries.');
