/**
 * maybeGarbageCollectAsync
 *
 * It might garbage collect, it might not. If it doesn't, that's ok.
 */
self.maybeGarbageCollectAsync = garbageCollect;

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
