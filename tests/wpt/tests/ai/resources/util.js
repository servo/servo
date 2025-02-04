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
