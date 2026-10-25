// META: script=performanceobservers.js

  async_test(function (t) {
    var observer = new PerformanceObserver(
        t.step_func(function (entryList, obs) {
          assert_unreached("This callback must not be invoked");
        })
      );
    observer.observe({entryTypes: ["mark", "measure", "navigation"]});
    observer.disconnect();
    self.performance.mark("mark1");
    self.performance.measure("measure1");
    t.step_timeout(function () {
      t.done();
    }, 2000);
  }, "disconnected callbacks must not be invoked");

  test(function () {
    var obs = new PerformanceObserver(function () { return true; });
    obs.disconnect();
    obs.disconnect();
  }, "disconnecting an unconnected observer is a no-op");

  async_test(function (t) {
    var observer = new PerformanceObserver(
        t.step_func(function (entryList, obs) {
          assert_unreached("This callback must not be invoked");
        })
      );
    observer.observe({entryTypes: ["mark"]});
    self.performance.mark("mark1");
    observer.disconnect();
    self.performance.mark("mark2");
    t.step_timeout(function () {
      t.done();
    }, 2000);
  }, "An observer disconnected after a mark must not have its callback invoked");

  test(function (t) {
    const observer = new PerformanceObserver(() => {});
    t.add_cleanup(() => self.performance.clearMarks("pending-before-disconnect"));
    observer.observe({entryTypes: ["mark"]});
    self.performance.mark("pending-before-disconnect");
    observer.disconnect();
    assert_array_equals(observer.takeRecords(), []);
  }, "takeRecords() returns no entries after disconnect()");

  async_test(function (t) {
    const observer = new PerformanceObserver(t.step_func_done(function (entryList) {
      assert_array_equals(entryList.getEntries().map(entry => entry.name),
                          ["mark-after-reobserve"]);
    }));
    t.add_cleanup(() => observer.disconnect());
    t.add_cleanup(() => self.performance.clearMarks("mark-before-disconnect"));
    t.add_cleanup(() => self.performance.clearMarks("mark-after-reobserve"));
    observer.observe({entryTypes: ["mark"]});
    self.performance.mark("mark-before-disconnect");
    observer.disconnect();
    observer.observe({entryTypes: ["mark"]});
    self.performance.mark("mark-after-reobserve");
  }, "Entries queued before disconnect() are not delivered after re-observing");
