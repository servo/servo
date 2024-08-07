test(() => {
  const source = new Observable(subscriber => {
    subscriber.next(1);
    subscriber.next(2);
    subscriber.next(3);
    subscriber.next(4);
    subscriber.complete();
  });

  const results = [];

  source
    .filter(value => value % 2 === 0)
    .subscribe({
      next: v => results.push(v),
      error: () => results.push("error"),
      complete: () => results.push("complete"),
    });

  assert_array_equals(results, [2, 4, "complete"]);
}, "filter(): Returned Observable filters out results based on predicate");

test(() => {
  const error = new Error("error while filtering");
  const results = [];
  let teardownCalled = false;

  const source = new Observable(subscriber => {
    subscriber.addTeardown(() => teardownCalled = true);
    subscriber.next(1);
    assert_true(teardownCalled, "Teardown called once filter unsubscribes due to error");
    assert_false(subscriber.active, "Unsubscription makes Subscriber inactive");
    results.push(subscriber.signal.reason);
    subscriber.next(2);
    subscriber.complete();
  });

  source
    .filter(() => {
      throw error;
    })
    .subscribe({
      next: v => results.push(v),
      error: e => results.push(e),
      complete: () => results.push("complete"),
    });

  assert_array_equals(results, [error, error]);
}, "filter(): Errors thrown in filter predicate are emitted to Observer error() handler");

test(() => {
  const source = new Observable(subscriber => {
    subscriber.next(1);
    subscriber.complete();
    subscriber.next(2);
  });

  let predicateCalls = 0;
  const results = [];
  source.filter(v => ++predicateCalls).subscribe({
    next: v => results.push(v),
    error: e => results.push(e),
    complete: () => results.push('complete'),
  });

  assert_equals(predicateCalls, 1, "Predicate is not called after complete()");
  assert_array_equals(results, [1, "complete"]);
}, "filter(): Passes complete() through from source Observable");

test(() => {
  const source = new Observable(subscriber => {
    subscriber.next(1);
    subscriber.error('error');
    subscriber.next(2);
  });

  let predicateCalls = 0;
  const results = [];
  source.filter(v => ++predicateCalls).subscribe({
    next: v => results.push(v),
    error: e => results.push(e),
    complete: () => results.push('complete'),
  });

  assert_equals(predicateCalls, 1, "Predicate is not called after error()");
  assert_array_equals(results, [1, "error"]);
}, "filter(): Passes error() through from source Observable");

test(() => {
  const results = [];
  const source = new Observable(subscriber => {
    subscriber.addTeardown(() => results.push('source teardown'));
    subscriber.signal.addEventListener('abort',
        () => results.push('source abort event'));

    subscriber.complete();
  });

  source.filter(() => results.push('filter predicate called')).subscribe({
    complete: () => results.push('filter observable complete'),
  });

  assert_array_equals(results,
      ['source abort event', 'source teardown', 'filter observable complete']);
}, "filter(): Upon source completion, source Observable teardown sequence " +
   "happens after downstream filter complete() is called");

test(() => {
  const source = new Observable(subscriber => {
    subscriber.next('value1');
    subscriber.next('value2');
    subscriber.next('value3');
  });

  const indices = [];
  source.filter((value, index) => indices.push(index)).subscribe();
  assert_array_equals(indices, [0, 1, 2]);
}, "filter(): Index is passed correctly to predicate");
