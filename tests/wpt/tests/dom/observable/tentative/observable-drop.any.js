test(() => {
  const source = new Observable(subscriber => {
    subscriber.next(1);
    subscriber.next(2);
    subscriber.next(3);
    subscriber.next(4);
    subscriber.complete();
  });

  const results = [];

  source.drop(2).subscribe({
    next: v => results.push(v),
    error: e => results.push(e),
    complete: () => results.push("complete"),
  });

  assert_array_equals(results, [3, 4, "complete"]);
}, "drop(): Observable should skip the first n values from the source " +
   "observable, then pass through the rest of the values and completion");

test(() => {
  const error = new Error('source error');
  const source = new Observable(subscriber => {
    subscriber.next(1);
    subscriber.next(2);
    subscriber.next(3);
    subscriber.next(4);
    subscriber.error(error);
  });

  const results = [];

  source.drop(2).subscribe({
    next: v => results.push(v),
    error: e => results.push(e),
    complete: () => results.push("complete"),
  });

  assert_array_equals(results, [3, 4, error]);
}, "drop(): Observable passes through errors from source Observable");

test(() => {
  const error = new Error('source error');
  const source = new Observable(subscriber => {
    subscriber.error(error);
    subscriber.next(1);
  });

  const results = [];

  source.drop(2).subscribe({
    next: v => results.push(v),
    error: e => results.push(e),
    complete: () => results.push("complete"),
  });

  assert_array_equals(results, [error]);
}, "drop(): Observable passes through errors from source observable even " +
   "before drop count is met");

test(() => {
  const source = new Observable(subscriber => {
    subscriber.next(1);
    subscriber.complete();
  });

  const results = [];

  source.drop(2).subscribe({
    next: v => results.push(v),
    error: e => results.push(e),
    complete: () => results.push("complete"),
  });

  assert_array_equals(results, ["complete"]);
}, "drop(): Observable passes through completions from source observable even " +
    "before drop count is met");

test(() => {
  let sourceTeardownCalled = false;
  const source = new Observable(subscriber => {
    subscriber.addTeardown(() => sourceTeardownCalled = true);
    subscriber.next(1);
    subscriber.next(2);
    subscriber.next(3);
    subscriber.next(4);
    subscriber.next(5);
    subscriber.complete();
  });

  const results = [];

  const controller = new AbortController();

  source.drop(2).subscribe({
    next: v => {
      results.push(v);
      if (v === 3) {
        controller.abort();
      }
    },
    error: (e) => results.push(e),
    complete: () => results.push("complete"),
  }, {signal: controller.signal});

  assert_true(sourceTeardownCalled,
      "Aborting outer observable unsubscribes the source observable");
  assert_array_equals(results, [3]);
}, "drop(): Unsubscribing from the Observable returned by drop() also " +
    "unsubscribes from the source Observable");

test(() => {
  const source = new Observable(subscriber => {
    subscriber.next(1);
    subscriber.next(2);
    subscriber.next(3);
    subscriber.complete();
  });

  const results = [];

  source.drop(0).subscribe({
    next: v => results.push(v),
    error: e => results.push(e),
    complete: () => results.push("complete"),
  });

  assert_array_equals(results, [1, 2, 3, "complete"],
      "Source Observable is mirrored");
}, "drop(): A drop amount of 0 simply mirrors the source Observable");

test(() => {
  const source = new Observable(subscriber => {
    subscriber.next(1);
    subscriber.next(2);
    subscriber.next(3);
    subscriber.complete();
  });

  const results = [];

  // Passing `-1` here is subject to the Web IDL integer conversion semantics,
  // which converts the drop amount to the maximum of `18446744073709551615`.
  source.drop(-1).subscribe({
    next: v => results.push(v),
    error: e => results.push(e),
    complete: () => results.push("complete"),
  });

  assert_array_equals(results, ["complete"], "Source Observable is mirrored");
}, "drop(): Passing negative value wraps to maximum value ");
