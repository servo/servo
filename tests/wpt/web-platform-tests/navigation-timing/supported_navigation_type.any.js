test(() => {
  if (typeof PerformanceObserver.supportedEntryTypes === "undefined")
    assert_unreached("supportedEntryTypes is not supported.");
  assert_true(PerformanceObserver.supportedEntryTypes.includes("navigation"),
    "There should be an entry 'navigation' in PerformanceObserver.supportedEntryTypes");
}, "supportedEntryTypes contains 'navigation'.");
