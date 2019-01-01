test(() => {
  if (typeof PerformanceObserver.supportedEntryTypes === "undefined")
    assert_unreached("supportedEntryTypes is not supported.");
  assert_true(PerformanceObserver.supportedEntryTypes.includes("paint"),
    "There should be an entry 'paint' in PerformanceObserver.supportedEntryTypes");
}, "supportedEntryTypes contains 'paint'.");
