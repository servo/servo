promise_test(async () => {
  let inactiveAfterB = false;
  const source = new Observable(subscriber => {
    subscriber.next("a");
    subscriber.next("b");
    inactiveAfterB = !subscriber.active;
    subscriber.next("c");
    subscriber.complete();
  });

  const value = await source.find((value) => value === "b");

  assert_equals(value, "b", "Promise resolves with the first value that passes the predicate");

  assert_true(inactiveAfterB, "subscriber is inactive after the first value that passes the predicate");
}, "find(): Promise resolves with the first value that passes the predicate");

promise_test(async () => {
  const source = new Observable(subscriber => {
    subscriber.next("a");
    subscriber.next("b");
    subscriber.next("c");
    subscriber.complete();
  });

  const value = await source.find(() => false);

  assert_equals(value, undefined, "Promise resolves with undefined if no value passes the predicate");
}, "find(): Promise resolves with undefined if no value passes the predicate");

promise_test(async t => {
  const error = new Error("error from source");
  const source = new Observable(subscriber => {
    subscriber.error(error);
  });

  promise_rejects_exactly(t, error, source.find(() => true), "Promise " +
      "rejects with the error emitted from the source Observable");
}, "find(): Promise rejects with the error emitted from the source Observable");

promise_test(async t => {
  const source = new Observable(subscriber => {
    subscriber.next("ignored");
  });

  const error = new Error("thrown from predicate");
  promise_rejects_exactly(t, error, source.find(() => {throw error}),
      "Promise rejects with any error thrown from the predicate");
}, "find(): Promise rejects with any error thrown from the predicate");

promise_test(async () => {
  let indices = [];

  const source = new Observable(subscriber => {
    subscriber.next("a");
    subscriber.next("b");
    subscriber.next("c");
    subscriber.complete();
  });

  const value = await source.find((value, index) => {
    indices.push(index);
    return false;
  });

  assert_equals(value, undefined, "Promise resolves with undefined if no value passes the predicate");

  assert_array_equals(indices, [0, 1, 2], "find(): Passes the index of the value to the predicate");
}, "find(): Passes the index of the value to the predicate");

promise_test(async t => {
  const controller = new AbortController();
  const source = new Observable(subscriber => {
    subscriber.next("a");
    subscriber.next("b");
    subscriber.next("c");
    subscriber.complete();
  });

  controller.abort();
  const promise = source.find(() => true, { signal: controller.signal });

  promise_rejects_dom(t, 'AbortError', promise, "Promise rejects with " +
      "DOMException when the signal is aborted");
}, "find(): Rejects with AbortError when the signal is aborted");
