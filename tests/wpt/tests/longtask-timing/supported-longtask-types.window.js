test(() => {
  assert_implements(typeof PerformanceObserver.supportedEntryTypes !== "undefined", 'supportedEntryTypes is not supported');
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

const entryType = "longtask";
promise_test(async () => {
  assert_implements(typeof PerformanceObserver.supportedEntryTypes !== "undefined", 'supportedEntryTypes is not supported');
  assert_implements(typeof PerformanceObserver.supportedEntryTypes.includes(entryType), `supportedEntryTypes does not include '${entryType}'`);
  await new Promise((resolve) => {
    new PerformanceObserver(function (list, observer) {
      observer.disconnect();
      resolve();
    }).observe({entryTypes: [entryType]});

    // Force the PerformanceEntry.
    syncWait(50);
  })
}, `'${entryType}' entries should be observable.`)
