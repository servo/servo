// META: title=Reporting: an observer whose report queue takeRecords() emptied isn't invoked
// META: script=/resources/testdriver.js
// META: script=/resources/testdriver-vendor.js

// https://w3c.github.io/reporting/#invoke-observers

promise_test(async test => {
  const calls = [];
  let takenReports;
  const second = new ReportingObserver(reports => calls.push(`second: ${reports.length}`));
  const first = new ReportingObserver(reports => {
    calls.push(`first: ${reports.length}`);
    // Empties the second observer's queue before its callback would run.
    takenReports = second.takeRecords();
  });
  let thirdCalled;
  const thirdPromise = new Promise(resolve => thirdCalled = resolve);
  const third = new ReportingObserver(reports => {
    calls.push(`third: ${reports.length}`);
    thirdCalled();
  });
  // Observers are notified in the order they started observing.
  first.observe();
  second.observe();
  third.observe();
  test.add_cleanup(() => [first, second, third].forEach(observer => observer.disconnect()));

  await test_driver.generate_test_report("Test message.");
  await thirdPromise;
  assert_equals(takenReports.length, 1, "takeRecords() returns the second observer's report");
  assert_array_equals(calls, ["first: 1", "third: 1"], "the second observer's callback isn't invoked");
}, "An observer whose report queue takeRecords() emptied isn't invoked");
