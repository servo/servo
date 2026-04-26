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
    CanvasTestType.PLACEHOLDER,
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

const DEFAULT_CANVAS_WIDTH = 300;
const DEFAULT_CANVAS_HEIGHT = 150;

var enabledTestTypeVariant = null;
setup(() => {
  const urlParams = new URLSearchParams(self.location.search);
  const testTypeVariant = urlParams.get('testType');
  if (testTypeVariant) {
    enabledTestTypeVariant = CanvasTestType[testTypeVariant.toUpperCase()];
    assert_true(!!enabledTestTypeVariant,
                `Unrecognized test type variant: ${testTypeVariant}`);
  }
});

function isTestTypeEnabled(testType) {
  return enabledTestTypeVariant === null || enabledTestTypeVariant === testType;
}

function createHTMLCanvasElement(width, height) {
  if (width === DEFAULT_CANVAS_WIDTH && height === DEFAULT_CANVAS_HEIGHT) {
    // Create a canvas with the default size.
    const canvas = document.createElement('canvas');
    assert_equals(canvas.width, width, 'Unexpected default canvas width.');
    assert_equals(canvas.height, height, 'Unexpected default canvas height.');
    return canvas;
  } else {
    // Create a canvas with the specified size from the start.
    const element = document.createElement('div');
    element.innerHTML = `<canvas width="${width}" height="${height}"></canvas>`;
    return element.firstElementChild;
  }
}

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
    testBody, description, {
      testTypes = DEFAULT_CANVAS_TEST_TYPES,
      width = DEFAULT_CANVAS_WIDTH,
      height = DEFAULT_CANVAS_HEIGHT,
    } = {}) {
  if (testTypes.includes(CanvasTestType.WORKER) &&
      isTestTypeEnabled(CanvasTestType.WORKER)) {
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

  if (testTypes.includes(CanvasTestType.HTML) &&
      isTestTypeEnabled(CanvasTestType.HTML)) {
    promise_test(async (test) => {
      if (!document.body) {
        document.documentElement.appendChild(document.createElement("body"));
      }
      const canvas = createHTMLCanvasElement(width, height);
      document.body.appendChild(canvas);
      await testBody(canvas, {test, canvasType: CanvasTestType.HTML});
      document.body.removeChild(canvas);
    }, 'HTMLCanvasElement: ' + description);
  }

  if (testTypes.includes(CanvasTestType.DETACHED_HTML) &&
      isTestTypeEnabled(CanvasTestType.DETACHED_HTML)) {
    promise_test((test) => {
      return testBody(createHTMLCanvasElement(width, height),
                      {test, canvasType: CanvasTestType.DETACHED_HTML});
    }, 'Detached HTMLCanvasElement: ' + description);
  }

  if (testTypes.includes(CanvasTestType.OFFSCREEN) &&
      isTestTypeEnabled(CanvasTestType.OFFSCREEN)) {
    promise_test((test) => {
      return testBody(new OffscreenCanvas(width, height),
                      {test, canvasType: CanvasTestType.OFFSCREEN});
    }, 'OffscreenCanvas: ' + description);
  }

  if (testTypes.includes(CanvasTestType.PLACEHOLDER) &&
      isTestTypeEnabled(CanvasTestType.PLACEHOLDER)) {
    promise_test(async (test) => {
      if (!document.body) {
        document.documentElement.appendChild(document.createElement("body"));
      }
      const placeholder = createHTMLCanvasElement(width, height);
      document.body.appendChild(placeholder);
      await testBody(placeholder.transferControlToOffscreen(),
                      {test, canvasType: CanvasTestType.PLACEHOLDER});
      document.body.removeChild(placeholder);
    }, 'PlaceholderCanvas: ' + description);
  }
}

/**
 * Run all the canvasPromiseTest from the current script in a worker.
 * If the tests depend on external scripts, these must be specified as a list
 * via the `dependencies` parameter so that the worker could load them.
 */
function runCanvasTestsInWorker({dependencies = []} = {}) {
  if (!isTestTypeEnabled(CanvasTestType.WORKER)) {
    return;
  }

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
    const allScripts = [
      // Forward `location.search` to the worker so that it could run the right
      // test variants. `location.search` is read-only in workers, so the whole
      // object has to be replaced.
      `var location = {search: '${self.location.search}'};`,
    ].concat(dependencyScripts)
     .concat([canvasTests, 'done();']);

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
