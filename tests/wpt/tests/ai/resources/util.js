const kValidAvailabilities =
    ['unavailable', 'downloadable', 'downloading', 'available'];
const kAvailableAvailabilities = ['downloadable', 'downloading', 'available'];

const kTestPrompt = 'Please write a sentence in English.';
const kTestContext = 'This is a test; this is only a test.';

const getId = (() => {
  let idCount = 0;
  return () => idCount++;
})();

// Takes an array of dictionaries mapping keys to value arrays, e.g.:
//   [ {Shape: ["Square", "Circle", undefined]}, {Count: [1, 2]} ]
// Returns an array of dictionaries with all value combinations, i.e.:
//  [ {Shape: "Square", Count: 1}, {Shape: "Square", Count: 2},
//    {Shape: "Circle", Count: 1}, {Shape: "Circle", Count: 2},
//    {Shape: undefined, Count: 1}, {Shape: undefined, Count: 2} ]
// Omits dictionary members when the value is undefined; supports array values.
function generateOptionCombinations(optionsSpec) {
  // 1. Extract keys from the input specification.
  const keys = optionsSpec.map(o => Object.keys(o)[0]);
  // 2. Extract the arrays of possible values for each key.
  const valueArrays = optionsSpec.map(o => Object.values(o)[0]);
  // 3. Compute the Cartesian product of the value arrays using reduce.
  const valueCombinations = valueArrays.reduce((accumulator, currentValues) => {
    // Init the empty accumulator (first iteration), with single-element
    // arrays.
    if (accumulator.length === 0) {
      return currentValues.map(value => [value]);
    }
    // Otherwise, expand existing combinations with current values.
    return accumulator.flatMap(
        existingCombo => currentValues.map(
            currentValue => [...existingCombo, currentValue]));
  }, []);

  // 4. Map each value combination to a result dictionary, skipping
  // undefined.
  return valueCombinations.map(combination => {
    const result = {};
    keys.forEach((key, index) => {
      if (combination[index] !== undefined) {
        result[key] = combination[index];
      }
    });
    return result;
  });
}

// The method should take the AbortSignal as an option and return a promise.
async function testAbortPromise(t, method) {
  // Test abort signal without custom error.
  {
    const controller = new AbortController();
    const promise = method(controller.signal);
    controller.abort();
    await promise_rejects_dom(t, 'AbortError', promise);

    // Using the same aborted controller will get the `AbortError` as well.
    const anotherPromise = method(controller.signal);
    await promise_rejects_dom(t, 'AbortError', anotherPromise);
  }

  // Test abort signal with custom error.
  {
    const err = new Error('test');
    const controller = new AbortController();
    const promise = method(controller.signal);
    controller.abort(err);
    await promise_rejects_exactly(t, err, promise);

    // Using the same aborted controller will get the same error as well.
    const anotherPromise = method(controller.signal);
    await promise_rejects_exactly(t, err, anotherPromise);
  }
};

async function testCreateMonitorWithAbortAt(
    t, loadedToAbortAt, method, options = {}) {
  const {promise: eventPromise, resolve} = Promise.withResolvers();
  let hadEvent = false;
  function monitor(m) {
    m.addEventListener('downloadprogress', e => {
      if (e.loaded != loadedToAbortAt) {
        return;
      }

      if (hadEvent) {
        assert_unreached(
            'This should never be reached since LanguageDetector.create() was aborted.');
        return;
      }

      resolve();
      hadEvent = true;
    });
  }

  const controller = new AbortController();

  const createPromise =
      method({...options, monitor, signal: controller.signal});

  await eventPromise;

  const err = new Error('test');
  controller.abort(err);
  await promise_rejects_exactly(t, err, createPromise);
}

async function testCreateMonitorWithAbort(t, method, options = {}) {
  await testCreateMonitorWithAbortAt(t, 0, method, options);
  await testCreateMonitorWithAbortAt(t, 1, method, options);
}

// The method should take the AbortSignal as an option and return a
// ReadableStream.
async function testAbortReadableStream(t, method) {
  // Test abort signal without custom error.
  {
    const controller = new AbortController();
    const stream = method(controller.signal);
    controller.abort();
    let writableStream = new WritableStream();
    await promise_rejects_dom(t, 'AbortError', stream.pipeTo(writableStream));

    // Using the same aborted controller will get the `AbortError` as well.
    await promise_rejects_dom(t, 'AbortError', new Promise(() => {
                                method(controller.signal);
                              }));
  }

  // Test abort signal with custom error.
  {
    const error = new DOMException('test', 'VersionError');
    const controller = new AbortController();
    const stream = method(controller.signal);
    controller.abort(error);
    let writableStream = new WritableStream();
    await promise_rejects_exactly(t, error, stream.pipeTo(writableStream));

    // Using the same aborted controller will get the same error.
    await promise_rejects_exactly(t, error, new Promise(() => {
                                    method(controller.signal);
                                  }));
  }
};

async function testMonitor(createFunc, options = {}) {
  let created = false;
  const progressEvents = [];
  function monitor(m) {
    m.addEventListener('downloadprogress', e => {
      // No progress events should be fired after `createFunc` resolves.
      assert_false(created);

      progressEvents.push(e);
    });
  }

  result = await createFunc({...options, monitor});
  created = true;

  assert_greater_than_equal(progressEvents.length, 2);
  assert_equals(progressEvents.at(0).loaded, 0);
  assert_equals(progressEvents.at(-1).loaded, 1);

  let lastProgressEventLoaded = -1;
  for (const progressEvent of progressEvents) {
    assert_equals(progressEvent.lengthComputable, true);
    assert_equals(progressEvent.total, 1);
    assert_less_than_equal(progressEvent.loaded, progressEvent.total);

    // `loaded` must be rounded to the nearest 0x10000th.
    assert_equals(progressEvent.loaded % (1 / 0x10000), 0);

    // Progress events should have monotonically increasing `loaded` values.
    assert_greater_than(progressEvent.loaded, lastProgressEventLoaded);
    lastProgressEventLoaded = progressEvent.loaded;
  }
  return result;
}

async function testCreateMonitorCallbackThrowsError(
    t, createFunc, options = {}) {
  const error = new Error('CreateMonitorCallback threw an error');
  function monitor(m) {
    m.addEventListener('downloadprogress', e => {
      assert_unreached(
          'This should never be reached since monitor throws an error.');
    });
    throw error;
  }

  await promise_rejects_exactly(t, error, createFunc({...options, monitor}));
}

function run_iframe_test(iframe, test_name) {
  const id = getId();
  iframe.contentWindow.postMessage({id, type: test_name}, '*');
  const {promise, resolve, reject} = Promise.withResolvers();
  window.onmessage = message => {
    if (message.data.id !== id) {
      return;
    }
    if (message.data.success) {
      resolve(message.data.success);
    } else {
      reject(message.data.err)
    }
  };
  return promise;
}

function load_iframe(src, permission_policy) {
  let iframe = document.createElement('iframe');
  const {promise, resolve} = Promise.withResolvers();
  iframe.onload = () => {
    resolve(iframe);
  };
  iframe.src = src;
  iframe.allow = permission_policy;
  document.body.appendChild(iframe);
  return promise;
}

async function createSummarizer(options = {}) {
  await test_driver.bless();
  return await Summarizer.create(options);
}

async function createWriter(options = {}) {
  await test_driver.bless();
  return await Writer.create(options);
}

async function createRewriter(options = {}) {
  await test_driver.bless();
  return await Rewriter.create(options);
}

async function createProofreader(options = {}) {
  await test_driver.bless();
  return await Proofreader.create(options);
}

async function testDestroy(t, createMethod, options, instanceMethods) {
  const instance = await createMethod(options);

  const promises = instanceMethods.map(method => method(instance));

  instance.destroy();

  promises.push(...instanceMethods.map(method => method(instance)));

  for (const promise of promises) {
    await promise_rejects_dom(t, 'AbortError', promise);
  }
}

async function testCreateAbort(t, createMethod, options, instanceMethods) {
  const controller = new AbortController();
  const instance = await createMethod({...options, signal: controller.signal});

  const promises = instanceMethods.map(method => method(instance));

  const error = new Error('The create abort signal was aborted.');
  controller.abort(error);

  promises.push(...instanceMethods.map(method => method(instance)));

  for (const promise of promises) {
    await promise_rejects_exactly(t, error, promise);
  }
}
