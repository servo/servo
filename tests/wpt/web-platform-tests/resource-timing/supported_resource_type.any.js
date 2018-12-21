test(() => {
  if (typeof PerformanceObserver.supportedEntryTypes === "undefined")
    assert_unreached("supportedEntryTypes is not supported.");
  assert_true(PerformanceObserver.supportedEntryTypes.includes("resource"),
    "There should be an entry 'resource' in PerformanceObserver.supportedEntryTypes");
}, "supportedEntryTypes contains 'resource'.");
