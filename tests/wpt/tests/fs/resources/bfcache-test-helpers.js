'use strict';

// Calls `createLock` with a file handle for `fileName`. Returns the lock if it
// succeeds. Returns undefined if it doesn't.
export async function tryToCreateLock(fileName, createLock) {
  const dir = await navigator.storage.getDirectory();
  const fileHandle = await dir.getFileHandle(fileName, {create: true});

  try {
    return await createLock(fileHandle);
  } catch {
    return undefined;
  }
}

// Returns a function that forwards `funcName` and the `args` passed to it to
// the `bfcache-test-worker.js` dedicated worker.
//
// Will create the dedicated worker if it doesn't already exist.
export const forwardToDedicatedWorker = (() => {
  let dedicatedWorker;

  // Returns a promise that resolves with the next dedicated worker result. Or
  // rejects if there is an error on the worker.
  function getNextDedicatedWorkerResult(dedicatedWorker) {
    return new Promise((resolve, reject) => {
      dedicatedWorker.addEventListener('message', ({data}) => {
        resolve(data);
      }, {once: true});
      dedicatedWorker.addEventListener('error', () => {
        reject(new Error('An error occurred on the dedicated worker.'));
      }, {once: true});
    });
  }

  return function(funcName) {
    return (...args) => {
      if (!dedicatedWorker) {
        dedicatedWorker = new Worker(
            `/fs/resources/bfcache-test-worker.js`, {type: 'module'});
      }

      dedicatedWorker.postMessage({funcName, args});
      return getNextDedicatedWorkerResult(dedicatedWorker);
    }
  }
})();
