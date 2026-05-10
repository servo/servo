function waitForConsoleEntries(t, {count, accept}) {
  return new Promise(resolve => {
    const entries = [];
    let done = false;

    const removeListener = test_driver.bidi.log.entry_added.on(entry => {
      if (done) return;
      if (entry.type !== "console") return;
      if (accept && !accept(entry)) return;

      entries.push(entry);
      if (entries.length === count) {
        done = true;
        removeListener();
        resolve(entries);
      }
    });

    t.add_cleanup(() => {
      if (!done) {
        done = true;
        removeListener();
      }
    });
  });
}
