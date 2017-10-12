// META: script=performanceobservers.js

  test(function () {
    var obs = new PerformanceObserver(function () { return true; });
    assert_throws(new TypeError(), function () {
      obs.observe({});
    });
    assert_throws(new TypeError(), function () {
      obs.observe({entryType: []});
    });
  }, "no entryTypes throws a TypeError");
  test(function () {
    var obs = new PerformanceObserver(function () { return true; });
    assert_throws(new TypeError(), function () {
      obs.observe({entryTypes: "mark"});
    });
  }, "entryTypes must be a sequence or throw a TypeError");

  test(function () {
    var obs = new PerformanceObserver(function () { return true; });
    obs.observe({entryTypes: []});
  }, "Empty sequence entryTypes is a no-op");

  test(function () {
    var obs = new PerformanceObserver(function () { return true; });
    obs.observe({entryTypes: ["this-cannot-match-an-entryType"]});
    obs.observe({entryTypes: ["marks","navigate", "resources"]});
  }, "Unknown entryTypes are no-op");

  test(function () {
    var obs = new PerformanceObserver(function () { return true; });
    obs.observe({entryTypes: ["mark","this-cannot-match-an-entryType"]});
    obs.observe({entryTypes: ["this-cannot-match-an-entryType","mark"]});
    obs.observe({entryTypes: ["mark"], others: true});
  }, "Filter unsupported entryType entryType names within the entryTypes sequence");

  async_test(function (t) {
    var finish = t.step_func(function () { t.done(); });
    var observer = new PerformanceObserver(
      function (entryList, obs) {
        var self = this;
        t.step(function () {
          assert_true(entryList instanceof PerformanceObserverEntryList, "first callback parameter must be a PerformanceObserverEntryList instance");
          assert_true(obs instanceof PerformanceObserver, "second callback parameter must be a PerformanceObserver instance");
          assert_equals(observer, self, "observer is the this value");
          assert_equals(observer, obs, "observer is second parameter");
          assert_equals(self, obs, "this and second parameter are the same");
          observer.disconnect();
          finish();
        });
      }
    );
    self.performance.clearMarks();
    observer.observe({entryTypes: ["mark"]});
    self.performance.mark("mark1");
  }, "Check observer callback parameter and this values");

  async_test(function (t) {
  var observer = new PerformanceObserver(
      t.step_func(function (entryList, obs) {
        checkEntries(entryList.getEntries(),
          [{ entryType: "measure", name: "measure1"}]);
        observer.disconnect();
        t.done();
      })
    );
    self.performance.clearMarks();
    observer.observe({entryTypes: ["mark"]});
    observer.observe({entryTypes: ["measure"]});
    self.performance.mark("mark1");
    self.performance.measure("measure1");
  }, "replace observer if already present");
