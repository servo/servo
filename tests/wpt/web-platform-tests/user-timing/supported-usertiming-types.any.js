test(() => {
  if (typeof PerformanceObserver.supportedEntryTypes === "undefined")
    assert_unreached("supportedEntryTypes is not supported.");
  const types = PerformanceObserver.supportedEntryTypes;
  assert_true(types.includes("mark"),
    "There should be 'mark' in PerformanceObserver.supportedEntryTypes");
  assert_true(types.includes("measure"),
    "There should be 'measure' in PerformanceObserver.supportedEntryTypes");
  assert_greater_than(types.indexOf("measure"), types.indexOf('mark'),
    "The 'measure' entry should appear after the 'mark' entry");
}, "supportedEntryTypes contains 'mark' and 'measure'.");
