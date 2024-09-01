promise_test(async t => {
  const source = new Observable(subscriber => {
    subscriber.next(1);
    subscriber.next(2);
    subscriber.next(3);
    t.step_timeout(() => subscriber.complete(), 0);
  });

  const reducerArguments = [];

  const promiseToResult = source.reduce((acc, value, index) => {
    reducerArguments.push([acc, value, index]);
    return acc + value;
  }, 0);

  // The reducer should be called immediately when the source emits a value.
  assert_equals(reducerArguments.length, 3);
  assert_array_equals(reducerArguments[0], [0, 1, 0]);
  assert_array_equals(reducerArguments[1], [1, 2, 1]);
  assert_array_equals(reducerArguments[2], [3, 3, 2]);

  const result = await promiseToResult;
  assert_equals(result, 6);
}, "reduce(): Reduces the values of the Observable, starting with the " +
   "initial seed value");

promise_test(async () => {
  let error = new Error('from the source');
  const source = new Observable(subscriber => {
    subscriber.next(1);
    subscriber.error(error);
  });

  let thrownError = null;
  try {
    await source.reduce((acc, value) => acc + value, 0);
  } catch (error) {
    thrownError = error;
  }

  assert_equals(thrownError, error);
}, "reduce(): Rejects if the source observable emits an error");

promise_test(async t => {
  const source = new Observable(subscriber => {
    subscriber.next(1);
    subscriber.next(2);
    subscriber.next(3);
    t.step_timeout(() => subscriber.complete(), 0);
  });

  const reducerArguments = [];

  const promiseToResult = source.reduce((acc, value, index) => {
    reducerArguments.push([acc, value, index]);
    return acc + value;
  });

  // The reducer should be called immediately when the source emits a value.
  assert_equals(reducerArguments.length, 2);
  assert_array_equals(reducerArguments[0], [1, 2, 1]);
  assert_array_equals(reducerArguments[1], [3, 3, 2]);

  const result = await promiseToResult;
  assert_equals(result, 6);
}, "reduce(): Seeds with the first value of the source, if no initial value " +
   "is provided");

promise_test(async () => {
  const logs = [];

  const source = new Observable(subscriber => {
    subscriber.addTeardown(() => logs.push('teardown'));
    logs.push('next 1');
    subscriber.next(1);
    logs.push('next 2');
    subscriber.next(2);
    logs.push('try to next 3');
    subscriber.next(3);
    logs.push('try to complete');
    subscriber.complete();
  });

  const error = new Error('from the reducer');
  let thrownError = null;

  try {
    await source.reduce((acc, value) => {
      if (value === 2) {
        logs.push('throw error');
        throw error;
      }
      return acc + value;
    }, 0);
  } catch (error) {
    thrownError = error;
  }

  assert_equals(thrownError, error);

  assert_array_equals(logs, [
    'next 1',
    'next 2',
    'throw error',
    'teardown',
    'try to next 3',
    'try to complete',
  ]);
}, "reduce(): Errors thrown in reducer reject the promise and abort the source");

promise_test(async () => {
  const source = new Observable(subscriber => {
    subscriber.complete();
  });

  const result = await source.reduce(() => 'reduced', 'seed');

  assert_equals(result, 'seed');
}, "reduce(): When source is empty, promise resolves with initial value");

promise_test(async () => {
  // This tests behavior that is analogous to `[].reduce(() => 'reduced')`,
  // which throws a TypeError.

  const source = new Observable(subscriber => {
    subscriber.complete();
  });

  let thrownError = null;
  try {
    await source.reduce(() => 'reduced');
  } catch (error) {
    thrownError = error;
  }

  assert_true(thrownError instanceof TypeError);
}, "reduce(): When source is empty, AND no seed value is provided, the " +
   "promise rejects with a TypeError");

promise_test(async t => {
  let tornDown = false;
  const source = new Observable((subscriber) => {
    subscriber.addTeardown(() => {
      tornDown = true;
    });
    // Waits forever.
  });

  const abortController = new AbortController();

  t.step_timeout(() => {
    abortController.abort();
    assert_true(tornDown);
  }, 0);

  let thrownError = null;
  try {
    await source.reduce(() => 'reduced', 'seed', { signal: abortController.signal });
  } catch (error) {
    thrownError = error;
  }

  assert_true(thrownError instanceof DOMException);
  assert_equals(thrownError.name, 'AbortError');
}, "reduce(): Reject with an AbortError if the subscription is aborted " +
   "before the source completes");
