/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/

function _defineProperty(obj, key, value) { if (key in obj) { Object.defineProperty(obj, key, { value: value, enumerable: true, configurable: true, writable: true }); } else { obj[key] = value; } return obj; }

export class AsyncMutex {
  constructor() {
    _defineProperty(this, "newestQueueItem", void 0);
  }

  // Run an async function with a lock on this mutex.
  // Waits until the mutex is available, locks it, runs the function, then releases it.
  async with(fn) {
    const p = (async () => {
      // If the mutex is locked, wait for the last thing in the queue before running.
      // (Everything in the queue runs in order, so this is after everything currently enqueued.)
      if (this.newestQueueItem) {
        await this.newestQueueItem;
      }

      return fn();
    })(); // Push the newly-created Promise onto the queue by replacing the old "newest" item.


    this.newestQueueItem = p; // And return so the caller can wait on the result.

    return p;
  }

}
//# sourceMappingURL=async_mutex.js.map