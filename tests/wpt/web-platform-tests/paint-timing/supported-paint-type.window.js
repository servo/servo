test(() => {
  if (typeof PerformanceObserver.supportedEntryTypes === "undefined")
    assert_unreached("supportedEntryTypes is not supported.");
  assert_true(PerformanceObserver.supportedEntryTypes.includes("paint"),
    "There should be an entry 'paint' in PerformanceObserver.supportedEntryTypes");
}, "supportedEntryTypes contains 'paint'.");

if (typeof PerformanceObserver.supportedEntryTypes !== "undefined") {
  const entryType = 'paint';
  if (PerformanceObserver.supportedEntryTypes.includes(entryType)) {
    promise_test(async() => {
      await new Promise((resolve) => {
        new PerformanceObserver(function (list, observer) {
          observer.disconnect();
          resolve();
        }).observe({entryTypes: [entryType]});

        // Force the PerformanceEntry.
        // Use `self` for Workers.
        if (self.document)
          document.head.parentNode.appendChild(document.createTextNode('foo'));
      })
    }, `'${entryType}' entries should be observable.`)
  }
}
