promise_test(async () => {
  const results = [];

  const source = new Observable(subscriber => {
    subscriber.addTeardown(() => results.push('teardown'));
    subscriber.next(1);
    results.push(subscriber.active ? 'active' : 'inactive');
    results.push(subscriber.signal.aborted ? 'aborted' : 'not aborted')

    // Ignored.
    subscriber.next(2);
    subscriber.complete();
  });

  const value = await source.first();

  assert_array_equals(results, ['teardown', 'inactive', 'aborted']);
  assert_equals(value, 1,
      "Promise resolves with the first value from the source Observable");
}, "first(): Promise resolves with the first value from the source Observable");

promise_test(async () => {
  const error = new Error("error from source");
  const source = new Observable(subscriber => {
    subscriber.error(error);
  });

  let rejection;
  try {
    await source.first();
  } catch (e) {
    rejection = e;
  }

  assert_equals(rejection, error, "Promise rejects with source Observable error");
}, "first(): Promise rejects with the error emitted from the source Observable");

promise_test(async () => {
  const source = new Observable(subscriber => {
    subscriber.complete();
  });

  let rejection;
  try {
    await source.first();
  } catch (e) {
    rejection = e;
  }

  assert_true(rejection instanceof RangeError,
      "Upon complete(), first() Promise rejects with RangeError");
  assert_equals(rejection.message, "No values in Observable");
}, "first(): Promise rejects with RangeError when source Observable " +
   "completes without emitting any values");

promise_test(async () => {
  const source = new Observable(subscriber => {});

  const controller = new AbortController();
  const promise = source.first({ signal: controller.signal });

  controller.abort();

  let rejection;
  try {
    await promise;
  } catch (e) {
    rejection = e;
  }

  assert_true(rejection instanceof DOMException,
      "Promise rejects with a DOMException for abortion");
  assert_equals(rejection.name, "AbortError",
      "Rejected with 'AbortError' DOMException");
  assert_equals(rejection.message, "signal is aborted without reason");
}, "first(): Aborting a signal rejects the Promise with an AbortError DOMException");

promise_test(async () => {
  const results = [];

  const source = new Observable(subscriber => {
    results.push("source subscribe");
    subscriber.addTeardown(() => results.push("source teardown"));
    subscriber.signal.addEventListener("abort", () => results.push("source abort"));
    results.push("before source next 1");
    subscriber.next(1);
    results.push("after source next 1");
  });

  results.push("calling first");
  const promise = source.first();

  assert_array_equals(results, [
    "calling first",
    "source subscribe",
    "before source next 1",
    "source abort",
    "source teardown",
    "after source next 1"
  ], "Array values after first() is called");

  const firstValue = await promise;
  results.push(`first resolved with: ${firstValue}`);

  assert_array_equals(results, [
    "calling first",
    "source subscribe",
    "before source next 1",
    "source abort",
    "source teardown",
    "after source next 1",
    "first resolved with: 1",
  ], "Array values after Promise is awaited");
}, "first(): Lifecycle");
