/**
 * Framework for executing tests with HTMLCanvasElement, main thread
 * OffscreenCanvas and worker OffscreenCanvas. Canvas tests are specified using
 * calls to `canvasPromiseTest`, which runs the test on the main thread, using
 * an HTML and an OffscreenCanvas. Calling `runCanvasTestsInWorker` at the
 * script level then re-execute the whole script in a worker, this time using
 * only OffscreenCanvas objects. Example usage:
 *
 * <script>
 * runCanvasTestsInWorker();
 *
 * canvasPromiseTest(async (canvas) => {
 *   // ...
 * }, "Sample test")
 * </script>
*/

/**
 * Enum listing all test types emitted by `canvasPromiseTest()`.
 */
const CanvasTestType = Object.freeze({
  HTML:   Symbol('html'),
  DETACHED_HTML:   Symbol('detached_html'),
  OFFSCREEN:  Symbol('offscreen'),
  PLACEHOLDER: Symbol('placeholder'),
  WORKER: Symbol('worker')
});

ALL_CANVAS_TEST_TYPES = Object.values(CanvasTestType);
DEFAULT_CANVAS_TEST_TYPES = [
    CanvasTestType.HTML,
    CanvasTestType.OFFSCREEN,
    CanvasTestType.WORKER,
];
HTML_CANVAS_ELEMENT_TEST_TYPES = [
    CanvasTestType.HTML,
    CanvasTestType.DETACHED_HTML,
];
OFFSCREEN_CANVAS_TEST_TYPES = [
    CanvasTestType.OFFSCREEN,
    CanvasTestType.WORKER,
];
MAIN_THREAD_CANVAS_TEST_TYPES = [
    CanvasTestType.HTML,
    CanvasTestType.DETACHED_HTML,
    CanvasTestType.OFFSCREEN,
    CanvasTestType.PLACEHOLDER,
];
WORKER_CANVAS_TEST_TYPES = [
    CanvasTestType.WORKER,
];

/**
 * Run `testBody` in a `promise_test` against multiple types of canvases. By
 * default, the test is executed against an HTMLCanvasElement, a main thread
 * OffscreenCanvas and a worker OffscreenCanvas, though `testTypes` can be used
 * only enable a subset of these. `testBody` must be a function accepting a
 * canvas as parameter and returning a promise that resolves on test completion.
 *
 * This function has two implementations. The version below runs the test on the
 * main thread and another version in `canvas-worker-test.js` runs it in a
 * worker. The worker invocation is launched by calling `runCanvasTestsInWorker`
 * at the script level.
 */
function canvasPromiseTest(
    testBody, description,
    {testTypes = DEFAULT_CANVAS_TEST_TYPES} = {}) {
  if (testTypes.includes(CanvasTestType.WORKER)) {
    setup(() => {
      const currentScript = document.currentScript;
      assert_true(
          currentScript.classList.contains('runCanvasTestsInWorkerInvoked'),
          'runCanvasTestsInWorker() must be called in the current script ' +
          'before calling canvasPromiseTest with CanvasTestType.WORKER test ' +
          'type, or else the test won\'t have worker coverage.');

      currentScript.classList.add('canvasWorkerTestAdded');
    });
  }

  if (testTypes.includes(CanvasTestType.HTML)) {
    promise_test(async () => {
      if (!document.body) {
        document.documentElement.appendChild(document.createElement("body"));
      }
      const canvas = document.createElement('canvas');
      document.body.appendChild(canvas);
      await testBody(canvas, {canvasType: CanvasTestType.HTML});
      document.body.removeChild(canvas);
    }, 'HTMLCanvasElement: ' + description);
  }

  if (testTypes.includes(CanvasTestType.DETACHED_HTML)) {
    promise_test(() => testBody(document.createElement('canvas'),
                                {canvasType: CanvasTestType.DETACHED_HTML}),
                 'Detached HTMLCanvasElement: ' + description);
  }

  if (testTypes.includes(CanvasTestType.OFFSCREEN)) {
    promise_test(() => testBody(new OffscreenCanvas(300, 150),
                                {canvasType: CanvasTestType.OFFSCREEN}),
                 'OffscreenCanvas: ' + description);
  }

  if (testTypes.includes(CanvasTestType.PLACEHOLDER)) {
    promise_test(async () => {
      if (!document.body) {
        document.documentElement.appendChild(document.createElement("body"));
      }
      const placeholder = document.createElement('canvas');
      document.body.appendChild(placeholder);
      await testBody(placeholder.transferControlToOffscreen(),
                     {canvasType: CanvasTestType.PLACEHOLDER});
    }, 'PlaceholderCanvas: ' + description);
  }
}

/**
 * Run all the canvasPromiseTest from the current script in a worker.
 * If the tests depend on external scripts, these must be specified as a list
 * via the `dependencies` parameter so that the worker could load them.
 */
function runCanvasTestsInWorker({dependencies = []} = {}) {
  const currentScript = document.currentScript;
  // Keep track of whether runCanvasTestsInWorker was invoked on the current
  // script. `canvasPromiseTest` will fail if `runCanvasTestsInWorker` hasn't
  // been called, to prevent accidentally omitting worker coverage.
  setup(() => {
    assert_false(
        currentScript.classList.contains('runCanvasTestsInWorkerInvoked'),
        'runCanvasTestsInWorker() can\'t be invoked twice on the same script.');
    currentScript.classList.add('runCanvasTestsInWorkerInvoked');
  });

  promise_setup(async () => {
    const allDeps = [
      '/resources/testharness.js',
      '/html/canvas/resources/canvas-promise-test.js',
      // canvas-promise-test-worker.js overrides parts of canvas-test.js.
      '/html/canvas/resources/canvas-promise-test-worker.js',
    ].concat(dependencies);

    const dependencyScripts =
       await Promise.all(allDeps.map(dep => fetch(dep).then(r => r.text())));
    const canvasTests = currentScript.textContent;
    const allScripts = dependencyScripts.concat([canvasTests, 'done();']);

    const workerBlob = new Blob(allScripts);
    const worker = new Worker(URL.createObjectURL(workerBlob));
    fetch_tests_from_worker(worker);
  });

  promise_setup(async () => {
    await new Promise(resolve => step_timeout(resolve, 0));
    assert_true(
        currentScript.classList.contains('canvasWorkerTestAdded'),
        'runCanvasTestsInWorker() should not be called if no ' +
        'canvasPromiseTest uses the CanvasTestType.WORKER test type.');
  });
}
