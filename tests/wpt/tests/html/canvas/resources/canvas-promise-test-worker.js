/** Worker script needed by canvas-test.js. */

/**
 * Worker version of `canvasPromiseTest()`, running `testBody` with an
 * `OffscreenCanvas` in a worker. For more details, see the main thread version
 * of this function in `canvas-test.js`.
 */
function canvasPromiseTest(
    testBody, description,
    {testTypes = Object.values(CanvasTestType)} = {}) {
  if (testTypes.includes(CanvasTestType.WORKER)) {
    promise_test(() => testBody(new OffscreenCanvas(300, 150),
                                {canvasType: CanvasTestType.WORKER}),
                'Worker: ' + description);
  }
}

/**
 * The function `runCanvasTestsInWorker()` in `canvas-test.js` re-executes the
 * current script in a worker. That script inevitably contain the call to
 * `runCanvasTestsInWorker()`, which triggered the whole thing. For that call
 * to succeed, the worker must have a definition for that function. There's
 * nothing to do here though, the script is already running in a worker.
 */
function runCanvasTestsInWorker() {}
