test(() => {
  const source = new Observable(subscriber => {
    subscriber.next(1);
    subscriber.next(2);
    subscriber.next(3);
    subscriber.complete();
  });

  const caughtObservable = source.catch(() => {
    assert_unreached("catch() is not called");
  });

  const results = [];

  caughtObservable.subscribe({
    next: value => results.push(value),
    complete: () => results.push('complete')
  });

  assert_array_equals(results, [1, 2, 3, 'complete']);
}, "catch(): Returns an Observable that is a pass-through for next()/complete()");

test(() => {
  let sourceError = new Error("from the source");
  const source = new Observable(subscriber => {
    subscriber.next(1);
    subscriber.next(2);
    subscriber.error(sourceError);
  });

  const caughtObservable = source.catch(error => {
    assert_equals(error, sourceError);
    return new Observable(subscriber => {
      subscriber.next(3);
      subscriber.complete();
    });
  });

  const results = [];

  caughtObservable.subscribe({
    next: value => results.push(value),
    complete: () => results.push("complete"),
  });

  assert_array_equals(results, [1, 2, 3, 'complete']);
}, "catch(): Handle errors from source and flatten to a new Observable");

test(() => {
  const sourceError = new Error("from the source");
  const source = new Observable(subscriber => {
    subscriber.next(1);
    subscriber.next(2);
    subscriber.error(sourceError);
  });

  const catchCallbackError = new Error("from the catch callback");
  const caughtObservable = source.catch(error => {
    assert_equals(error, sourceError);
    throw catchCallbackError;
  });

  const results = [];

  caughtObservable.subscribe({
    next: value => results.push(value),
    error: error => {
      results.push(error);
    },
    complete: () => results.push('complete'),
  });

  assert_array_equals(results, [1, 2, catchCallbackError]);
}, "catch(): Errors thrown in the catch() callback are sent to the consumer's error handler");

test(() => {
  // A common use case is logging and keeping the stream alive.
  const source = new Observable(subscriber => {
    subscriber.next(1);
    subscriber.next(2);
    subscriber.next(3);
    subscriber.complete();
  });

  const flatteningError = new Error("from the flattening operation");
  function errorsOnTwo(value) {
    return new Observable(subscriber => {
      if (value === 2) {
        subscriber.error(flatteningError);
      } else {
        subscriber.next(value);
        subscriber.complete();
      }
    });
  }

  const results = [];

  source.flatMap(value => errorsOnTwo(value)
    .catch(error => {
      results.push(error);
      // This empty array converts to an Observable which automatically
      // completes.
      return [];
    })
  ).subscribe({
    next: value => results.push(value),
    complete: () => results.push("complete")
  });

  assert_array_equals(results, [1, flatteningError, 3, "complete"]);
}, "catch(): CatchHandler can return an empty iterable");

promise_test(async () => {
  const sourceError = new Error("from the source");
  const source = new Observable(subscriber => {
    subscriber.next(1);
    subscriber.next(2);
    subscriber.error(sourceError);
  });

  const caughtObservable = source.catch(error => {
    assert_equals(error, sourceError);
    return Promise.resolve(error.message);
  });

  const results = await caughtObservable.toArray();

  assert_array_equals(results, [1, 2, "from the source"]);
}, "catch(): CatchHandler can return a Promise");

promise_test(async () => {
  const source = new Observable(subscriber => {
    subscriber.next(1);
    subscriber.next(2);
    subscriber.error(new Error('from the source'));
  });

  const caughtObservable = source.catch(async function* (error) {
    assert_true(error instanceof Error);
    assert_equals(error.message, 'from the source');
    yield 3;
  });

  const results = await caughtObservable.toArray();

  assert_array_equals(results, [1, 2, 3], 'catch(): should handle returning an observable');
}, 'catch(): should handle returning an async iterable');

test(() => {
  const sourceError = new Error("from the source");
  const source = new Observable(subscriber => {
    subscriber.next(1);
    subscriber.next(2);
    subscriber.error(sourceError);
  });

  const caughtObservable = source.catch(error => {
    assert_equals(error, sourceError);
    // Primitive values like this are not convertible to an Observable, via the
    // `from()` semantics.
    return 3;
  });

  const results = [];

  caughtObservable.subscribe({
    next: value => results.push(value),
    error: error => {
      assert_true(error instanceof TypeError);
      results.push("TypeError");
    },
    complete: () => results.push("complete"),
  });

  assert_array_equals(results, [1, 2, "TypeError"]);
}, "catch(): CatchHandler emits an error if the value returned is not " +
   "convertible to an Observable");

test(() => {
  const source = new Observable(subscriber => {
    susbcriber.error(new Error("from the source"));
  });

  const results = [];

  const innerSubscriptionError = new Error("CatchHandler subscription error");
  const catchObservable = source.catch(() => {
    results.push('CatchHandler invoked');
    return new Observable(subscriber => {
      throw innerSubscriptionError;
    });
  });

  catchObservable.subscribe({
    error: e => {
      results.push(e);
    }
  });

  assert_array_equals(results, ['CatchHandler invoked', innerSubscriptionError]);
}, "catch(): CatchHandler returns an Observable that throws immediately on " +
   "subscription");

// This test asserts that the relationship between (a) the AbortSignal passed
// into `subscribe()` and (b) the AbortSignal associated with the Observable
// returned from `catch()`'s CatchHandler is not a "dependent" relationship.
// This is important because Observables have moved away from the "dependent
// abort signal" infrastructure in https://github.com/WICG/observable/pull/154,
// and this test asserts so.
//
// Here are all of the associated Observables and signals in this test:
// 1. Raw outer signal passed into `subscribe()`
// 2. catchObservable's inner Subscriber's signal
//    a. Per the above PR, and Subscriber's initialization logic [1], this
//       signal is set to abort in response to (1)'s abort algorithms. This
//       means its "abort" event gets fired before (1)'s.
// 3. Inner CatchHandler-returned Observable's Subscriber's signal
//    a. Also per [1], this is set to abort in response to (2)'s abort
//       algorithms, since we subscribe to this "inner Observable" with (2)'s
//       signal as the `SubscribeOptions#signal`.
//
// (1), (2), and (3) above all form an abort chain:
// (1) --> (2) --> (3)
//
// …such that when (1) aborts, its abort algorithms immediately abort (2),
// whose abort algorithms immediately abort (3). Finally on the way back up the
// chain, (3)'s `abort` event is fired, (2)'s `abort` event is fired, and then
// (1)'s `abort` event is fired. This ordering of abort events is what this test
// ensures.
//
// [1]: https://wicg.github.io/observable/#ref-for-abortsignal-add
test(() => {
  const results = [];
  const source = new Observable(subscriber =>
      susbcriber.error(new Error("from the source")));

  const catchObservable = source.catch(() => {
    return new Observable(subscriber => {
      subscriber.addTeardown(() => results.push('inner teardown'));
      subscriber.signal.addEventListener('abort',
          e => results.push('inner signal abort'));

      // No values or completion. We'll just wait for the subscriber to abort
      // its subscription.
    });
  });

  const ac = new AbortController();
  ac.signal.addEventListener('abort', e => results.push('outer signal abort'));
  catchObservable.subscribe({}, {signal: ac.signal});
  ac.abort();

  assert_array_equals(results, ['inner signal abort', 'inner teardown', 'outer signal abort']);
}, "catch(): Abort order between outer AbortSignal and inner CatchHandler subscriber's AbortSignal");
