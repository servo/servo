const kTestPrompt = 'Please write a sentence in English.';

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

// The method should take the AbortSignal as an option and return a ReadableStream.
const testAbortReadableStream = async (t, method) => {
  // Test abort signal without custom error.
  {
    const controller = new AbortController();
    const stream = method(controller.signal);
    controller.abort();
    let writableStream = new WritableStream();
    await promise_rejects_dom(
      t, "AbortError", stream.pipeTo(writableStream)
    );

    // Using the same aborted controller will get the `AbortError` as well.
    await promise_rejects_dom(
      t, "AbortError", new Promise(() => { method(controller.signal); })
    );
  }

  // Test abort signal with custom error.
  {
    const error = new DOMException("test", "VersionError");
    const controller = new AbortController();
    const stream = method(controller.signal);
    controller.abort(error);
    let writableStream = new WritableStream();
    await promise_rejects_exactly(
      t, error,
      stream.pipeTo(writableStream)
    );

    // Using the same aborted controller will get the same error.
    await promise_rejects_exactly(
      t, error, new Promise(() => { method(controller.signal); })
    );
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

  await createFunc({...options, monitor});
  created = true;

  assert_greater_than_equal(progressEvents.length, 2);
  assert_equals(progressEvents.at(0).loaded, 0);
  assert_equals(progressEvents.at(-1).loaded, 1);

  let lastProgressEventLoaded = -1;
  for (const progressEvent of progressEvents) {
    assert_equals(progressEvent.total, 1);
    assert_less_than_equal(progressEvent.loaded, progressEvent.total);

    // Progress events should have monotonically increasing `loaded` values.
    assert_greater_than(progressEvent.loaded, lastProgressEventLoaded);
    lastProgressEventLoaded = progressEvent.loaded;
  }
}
