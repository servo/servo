/**
 * maybeGarbageCollectAsync
 *
 * It might garbage collect, it might not. If it doesn't, that's ok.
 *
 * Based on "(default export)" in
 * https://github.com/web-platform-tests/wpt/pull/22835/files#diff-fba53ea423a12f40917f41ba4ffadf1e, and "$262.gc()"
 * defined in https://github.com/tc39/test262/blob/main/INTERPRETING.md
 *
 *
 * @return {undefined}
 */
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
  }
  /* eslint-disable no-console */
  console.warn('Tests are running without the ability to do manual ' +
               'garbage collection. They will still work, but ' +
               'coverage will be suboptimal.');
  /* eslint-enable no-console */
}

/**
 * maybeGarbageCollectKeptObjectsAsync
 *
 * Based on "asyncGCDeref" in https://github.com/tc39/test262/blob/master/harness/async-gc.js
 *
 * @return {Promise} Resolves to a trigger if ClearKeptObjects
 *                   exists to provide one
 */
async function maybeGarbageCollectKeptObjectsAsync() {
  let trigger;

  if (typeof ClearKeptObjects === 'function') {
    trigger = ClearKeptObjects();
  }

  await maybeGarbageCollectAsync();

  return trigger;
}

/**
 * maybeGarbageCollectAndCleanupAsync
 *
 * Based on "asyncGC" in https://github.com/tc39/test262/blob/master/harness/async-gc.js
 *
 * @return {undefined}
 */
async function maybeGarbageCollectAndCleanupAsync(...targets) {
  let finalizationRegistry = new FinalizationRegistry(() => {});
  let length = targets.length;

  for (let target of targets) {
    finalizationRegistry.register(target, 'target');
    target = null;
  }

  targets = null;

  await 'tick';
  await maybeGarbageCollectKeptObjectsAsync();

  let names = [];

  finalizationRegistry.cleanupSome(name => names.push(name));

  if (names.length !== length) {
    throw maybeGarbageCollectAndCleanupAsync.NOT_COLLECTED;
  }
}

maybeGarbageCollectAndCleanupAsync.NOT_COLLECTED = Symbol('Object was not collected');

/**
 * resolveGarbageCollection
 *
 * Based on "resolveAsyncGC" in https://github.com/tc39/test262/blob/master/harness/async-gc.js
 *
 * @param  {Error} error An error object.
 * @return {undefined}
 */
function resolveGarbageCollection(error) {
  if (error && error !== maybeGarbageCollectAndCleanupAsync.NOT_COLLECTED) {
    throw error;
  }
}
