test(() => {
  const results = [];
  const indices = [];
  const source = new Observable((subscriber) => {
    subscriber.next(1);
    subscriber.next(2);
    subscriber.next(3);
    subscriber.complete();
  });

  const mapped = source.map((value, i) => {
    indices.push(i);
    return value * 2;
  });

  assert_true(mapped instanceof Observable, "map() returns an Observable");

  assert_array_equals(results, [], "Does not map until subscribed (values)");
  assert_array_equals(indices, [], "Does not map until subscribed (indices)");

  mapped.subscribe({
    next: (value) => results.push(value),
    error: () => results.push('error'),
    complete: () => results.push('complete'),
  });

  assert_array_equals(results, [2, 4, 6, 'complete']);
  assert_array_equals(indices, [0, 1, 2]);
}, "map(): Maps values correctly");

test(() => {
  const error = new Error("error");
  const results = [];
  let teardownCalled = false;

  const source = new Observable((subscriber) => {
    subscriber.addTeardown(() => teardownCalled = true);

    subscriber.next(1);
    assert_false(teardownCalled,
        "Teardown not called until until map unsubscribes due to error");
    subscriber.next(2);
    assert_true(teardownCalled, "Teardown called once map unsubscribes due to error");
    assert_false(subscriber.active, "Unsubscription makes Subscriber inactive");
    subscriber.next(3);
    subscriber.complete();
  });

  const mapped = source.map((value) => {
    if (value === 2) {
      throw error;
    }
    return value * 2;
  });

  mapped.subscribe({
    next: (value) => results.push(value),
    error: (error) => results.push(error),
    complete: () => results.push("complete"),
  });

  assert_array_equals(results, [2, error],
      "Mapper errors are emitted to Observer error() handler");
}, "map(): Mapper errors are emitted to Observer error() handler");

test(() => {
  const source = new Observable(subscriber => {
    subscriber.next(1);
    subscriber.complete();
    subscriber.next(2);
  });

  let mapperCalls = 0;
  const results = [];
  source.map(v => {
    mapperCalls++;
    return v * 2;
  }).subscribe({
    next: v => results.push(v),
    error: e => results.push(e),
    complete: () => results.push('complete'),
  });

  assert_equals(mapperCalls, 1, "Mapper is not called after complete()");
  assert_array_equals(results, [2, "complete"]);
}, "map(): Passes complete() through from source Observable");

test(() => {
  const source = new Observable(subscriber => {
    subscriber.next(1);
    subscriber.error('error');
    subscriber.next(2);
  });

  let mapperCalls = 0;
  const results = [];
  source.map(v => {
    mapperCalls++;
    return v * 2;
  }).subscribe({
    next: v => results.push(v),
    error: e => results.push(e),
    complete: () => results.push('complete'),
  });

  assert_equals(mapperCalls, 1, "Mapper is not called after error()");
  assert_array_equals(results, [2, "error"]);
}, "map(): Passes error() through from source Observable");

// This is mostly ensuring that the ordering in
// https://wicg.github.io/observable/#dom-subscriber-complete is consistent.
//
// That is, the `Subscriber#complete()` method *first* closes itself and signals
// abort on its own `Subscriber#signal()` and *then* calls whatever supplied
// completion algorithm exists. In the case of `map()`, the "supplied completion
// algorithm" is simply a set of internal observer steps that call
// `Subscriber#complete()` on the *outer* mapper's Observer. This means the
// outer Observer is notified of completion *after* the source Subscriber's
// signal is aborted / torn down.
test(() => {
  const results = [];
  const source = new Observable(subscriber => {
    subscriber.addTeardown(() => results.push('source teardown'));
    subscriber.signal.addEventListener('abort',
        () => results.push('source abort event'));

    subscriber.complete();
  });

  source.map(() => results.push('mapper called')).subscribe({
    complete: () => results.push('map observable complete'),
  });

  assert_array_equals(results,
      ['source abort event', 'source teardown', 'map observable complete']);
}, "map(): Upon source completion, source Observable teardown sequence " +
   "happens before downstream mapper complete() is called");

test(() => {
  const results = [];
  let sourceSubscriber = null;
  const source = new Observable(subscriber => {
    subscriber.addTeardown(() => results.push('source teardown'));
    sourceSubscriber = subscriber;

    subscriber.next(1);
  });

  const controller = new AbortController();
  source.map(v => v * 2).subscribe({
    next: v => {
      results.push(v);

      // Triggers unsubscription to `source`.
      controller.abort();

      // Does nothing, since `source` is already torn down.
      sourceSubscriber.next(100);
    },
    complete: () => results.push('mapper complete'),
    error: e => results.push('mapper error'),
  }, {signal: controller.signal});

  assert_array_equals(results, [2, 'source teardown']);
}, "map(): Map observable unsubscription causes source Observable " +
   "unsubscription. Mapper Observer's complete()/error() are not called");
