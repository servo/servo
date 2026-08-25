test(() => {
  const results = [];
  const source = new Observable(subscriber => {
    subscriber.addTeardown(() => results.push("source teardown"));
    subscriber.next(1);
    subscriber.next(2);
    subscriber.next(3);
    subscriber.complete();
  });

  const result = source.take(2);

  result.subscribe({
    next: v => results.push(v),
    error: e => results.push(e),
    complete: () => results.push("complete"),
  });

  assert_array_equals(results, [1, 2, "source teardown", "complete"]);
}, "take(): Takes the first N values from the source observable, then completes");

test(() => {
  const results = [];
  const source = new Observable(subscriber => {
    subscriber.addTeardown(() => results.push("source teardown"));
    subscriber.next(1);
    subscriber.next(2);
    subscriber.next(3);
    subscriber.complete();
  });

  const result = source.take(5);

  result.subscribe({
    next: v => results.push(v),
    error: e => results.push(e),
    complete: () => results.push("complete"),
  });

  assert_array_equals(results, [1, 2, 3, "source teardown", "complete"],
      "complete() is immediately forwarded");
}, "take(): Forwards complete()s that happen before the take count is met, " +
   "and unsubscribes from source Observable");

test(() => {
  const results = [];
  const error = new Error('source error');
  const source = new Observable(subscriber => {
    subscriber.next(1);
    subscriber.error(error);
  });

  const result = source.take(100);

  result.subscribe({
    next: v => results.push(v),
    error: e => results.push(e),
    complete: () => results.push("complete"),
  });

  assert_array_equals(results, [1, error], "Errors are forwarded");
}, "take(): Should forward errors from the source observable");

test(() => {
  const results = [];
  const source = new Observable((subscriber) => {
    results.push("source subscribe");
    subscriber.next(1);
    subscriber.next(2);
    subscriber.next(3);
    subscriber.complete();
  });

  const result = source.take(0);

  result.subscribe({
    next: v => results.push(v),
    error: e => results.push(e),
    complete: () => results.push("complete"),
  });

  assert_array_equals(results, ["complete"]);
}, "take(): take(0) should not subscribe to the source observable, and " +
   "should return an observable that immediately completes");

test(() => {
  const results = [];
  const source = new Observable((subscriber) => {
    results.push("source subscribe");
    subscriber.next(1);
    subscriber.next(2);
    subscriber.next(3);
    subscriber.complete();
  });

  // Per WebIDL, `-1` passed into an `unsigned long long` gets wrapped around
  // into the maximum value (18446744073709551615), which means the `result`
  // Observable captures everything that `source` does.
  const result = source.take(-1);

  result.subscribe({
    next: v => results.push(v),
    error: e => results.push(e),
    complete: () => results.push("complete"),
  });

  assert_array_equals(results, ["source subscribe", 1, 2, 3, "complete"]);
}, "take(): Negative count is treated as maximum value");

// This tests a regression in Chromium's implementation. In ref-counted
// producers, when Subscriber#next() is called, the Subscriber iterates over all
// of its "internal observers" [1] and calls "next" on them. However, "next" can
// complete the subscription, and modify the "internal observers" list while
// Subscriber is iterating over it. This mutation-during-iteration caused a
// crash regression in Chromium, which this test covers.
//
// [1]: https://wicg.github.io/observable/#subscriber-internal-observers
promise_test(async () => {
  async function* asyncNumbers() {
    yield* [1,2,3,4];
  }

  const source = Observable.from(asyncNumbers());
  const results = [];

  source.take(1).toArray().then(result => results.push(result));
  await source.take(3).toArray().then(result => results.push(result));

  assert_equals(results.length, 2);
  assert_array_equals(results[0], [1]);
  assert_array_equals(results[1], [1, 2, 3]);
}, "take(): No crash when take(1) unsubscribes from its source when next() " +
   "is called, and the Subscriber iterates over the rest of the Observables");
