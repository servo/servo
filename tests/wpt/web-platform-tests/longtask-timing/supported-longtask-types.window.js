test(() => {
  if (typeof PerformanceObserver.supportedEntryTypes === "undefined")
    assert_unreached("supportedEntryTypes is not supported.");
  const types = PerformanceObserver.supportedEntryTypes;
  assert_true(types.includes("longtask"),
    "There should be 'longtask' in PerformanceObserver.supportedEntryTypes");
  assert_false(types.includes("taskattribution"),
    "There should NOT be 'taskattribution' in PerformanceObserver.supportedEntryTypes");
}, "supportedEntryTypes contains 'longtask' but not 'taskattribution'.");

function syncWait(waitDuration) {
  if (waitDuration <= 0)
    return;

  const startTime = performance.now();
  let unused = '';
  for (let i = 0; i < 10000; i++)
    unused += '' + Math.random();

  return syncWait(waitDuration - (performance.now() - startTime));
}

if (typeof PerformanceObserver.supportedEntryTypes !== "undefined") {
  const entryType = "longtask";
  if (PerformanceObserver.supportedEntryTypes.includes(entryType)) {
    promise_test(async () => {
      await new Promise((resolve) => {
        new PerformanceObserver(function (list, observer) {
          observer.disconnect();
          resolve();
        }).observe({entryTypes: [entryType]});

        // Force the PerformanceEntry.
        syncWait(50);
      })
    }, `'${entryType}' entries should be observable.`)
  }
}
