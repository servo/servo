async function maybeGarbageCollectAsync() {
  if (typeof TestUtils !== 'undefined' && TestUtils.gc) {
    await TestUtils.gc();
  } else if (self.gc) {
    // Use --expose_gc for V8 (and Node.js)
    // to pass this flag at chrome launch use: --js-flags="--expose-gc"
    // Exposed in SpiderMonkey shell as well
    await self.gc();
  } else if (self.GCController) {
    // Present in some WebKit development environments
    await GCController.collect();
  } else {
    /* eslint-disable no-console */
    console.warn('Tests are running without the ability to do manual ' +
                 'garbage collection. They will still work, but ' +
                 'coverage will be suboptimal.');
    /* eslint-enable no-console */
  }
}
