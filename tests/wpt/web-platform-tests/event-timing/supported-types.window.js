test(() => {
  if (typeof PerformanceObserver.supportedEntryTypes === "undefined")
    assert_unreached("supportedEntryTypes is not supported.");
  const types = PerformanceObserver.supportedEntryTypes;
  assert_true(types.includes("firstInput"),
    "There should be 'firstInput' in PerformanceObserver.supportedEntryTypes");
  assert_true(types.includes("event"),
    "There should be 'event' in PerformanceObserver.supportedEntryTypes");
  assert_greater_than(types.indexOf("firstInput"), types.indexOf('event'),
    "The 'firstInput' entry should appear after the 'event' entry");
}, "supportedEntryTypes contains 'event' and 'firstInput'.");
