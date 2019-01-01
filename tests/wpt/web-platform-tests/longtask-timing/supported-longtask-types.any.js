test(() => {
  if (typeof PerformanceObserver.supportedEntryTypes === "undefined")
    assert_unreached("supportedEntryTypes is not supported.");
  const types = PerformanceObserver.supportedEntryTypes;
  assert_true(types.includes("longtask"),
    "There should be 'longtask' in PerformanceObserver.supportedEntryTypes");
  assert_false(types.includes("taskattribution"),
    "There should NOT be 'taskattribution' in PerformanceObserver.supportedEntryTypes");
}, "supportedEntryTypes contains 'longtask' but not 'taskattribution'.");
