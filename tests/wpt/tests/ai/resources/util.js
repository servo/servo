const kValidAvailabilities =
    ['unavailable', 'downloadable', 'downloading', 'available'];
const kAvailableAvailabilities = ['downloadable', 'downloading', 'available'];

const kTestPrompt = 'Please write a sentence in English.';

// Takes an array of dictionaries mapping keys to value arrays, e.g.:
//   [ {Shape: ["Square", "Circle", undefined]}, {Count: [1, 2]} ]
// Returns an array of dictionaries with all value combinations, i.e.:
//  [ {Shape: "Square", Count: 1}, {Shape: "Square", Count: 2},
//    {Shape: "Circle", Count: 1}, {Shape: "Circle", Count: 2},
//    {Shape: undefined, Count: 1}, {Shape: undefined, Count: 2} ]
// Omits dictionary members when the value is undefined; supports array values.
const generateOptionCombinations =
    (optionsSpec) => {
      // 1. Extract keys from the input specification.
      const keys = optionsSpec.map(o => Object.keys(o)[0]);
      // 2. Extract the arrays of possible values for each key.
      const valueArrays = optionsSpec.map(o => Object.values(o)[0]);
      // 3. Compute the Cartesian product of the value arrays using reduce.
      const valueCombinations =
          valueArrays.reduce((accumulator, currentValues) => {
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
const testAbortPromise = async (t, method) => {
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

// The method should take the AbortSignal as an option and return a
// ReadableStream.
const testAbortReadableStream = async (t, method) => {
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
