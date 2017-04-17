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
    assert_throws(new TypeError(), function () {
      obs.observe({entryTypes: []});
    });
    assert_throws(new TypeError(), function () {
      obs.observe({entryTypes: ["this-cannot-match-an-entryType"]});
    });
    assert_throws(new TypeError(), function () {
      obs.observe({entryTypes: ["marks","navigate", "resources"]});
    });
  }, "Empty sequence entryTypes throws a TypeError");

  test(function () {
    var obs = new PerformanceObserver(function () { return true; });
    obs.observe({entryTypes: ["mark","this-cannot-match-an-entryType"]});
    obs.observe({entryTypes: ["this-cannot-match-an-entryType","mark"]});
    obs.observe({entryTypes: ["mark"], others: true});
  }, "Filter unsupported entryType entryType names within the entryTypes sequence");

  async_test(function (t) {
  var observer = new PerformanceObserver(
      t.step_func(function (entryList, obs) {
        assert_equals(observer, obs, "observer is second parameter");
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
