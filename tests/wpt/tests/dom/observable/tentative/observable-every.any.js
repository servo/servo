promise_test(async () => {
  const source = new Observable(subscriber => {
    subscriber.next("good");
    subscriber.next("good");
    subscriber.next("good");
    subscriber.complete();
  });

  const result = await source.every((value) => value === "good");

  assert_true(result, "Promise resolves with true if all values pass the predicate");
}, "every(): Promise resolves to true if all values pass the predicate");

promise_test(async () => {
  const source = new Observable(subscriber => {
    subscriber.next("good");
    subscriber.next("good");
    subscriber.next("bad");
    subscriber.complete();
  });

  const result = await source.every((value) => value === "good");

  assert_false(result, "Promise resolves with false if any value fails the predicate");
}, "every(): Promise resolves to false if any value fails the predicate");

promise_test(async () => {
  let tornDown = false;
  let subscriberActiveAfterFailingPredicate = true;
  const source = new Observable(subscriber => {
    subscriber.addTeardown(() => tornDown = true);
    subscriber.next("good");
    subscriber.next("good");
    subscriber.next("bad");
    subscriberActiveAfterFailingPredicate = subscriber.active;
    subscriber.next("good");
    subscriber.complete();
  });

  const result = await source.every((value) => value === "good");

  assert_false(result, "Promise resolves with false if any value fails the predicate");
  assert_false(subscriberActiveAfterFailingPredicate,
      "Subscriber becomes inactive because every() unsubscribed");
}, "every(): Abort the subscription to the source if the predicate does not pass");

promise_test(async () => {
  const logs = [];

  const source = createTestSubject({
    onSubscribe: () => logs.push("subscribed to source"),
    onTeardown: () => logs.push("teardown"),
  });

  const resultPromise = source.every((value, index) => {
    logs.push(`Predicate called with ${value}, ${index}`);
    return true;
  });

  let promiseResolved = false;

  resultPromise.then(() => promiseResolved = true);

  assert_array_equals(logs, ["subscribed to source"],
      "calling every() subscribes to the source immediately");

  source.next("a");
  assert_array_equals(logs, [
    "subscribed to source",
    "Predicate called with a, 0"
  ], "Predicate called with the value and the index");

  source.next("b");
  assert_array_equals(logs, [
    "subscribed to source",
    "Predicate called with a, 0",
    "Predicate called with b, 1",
  ], "Predicate called with the value and the index");

  // wait a tick, just to prove that you have to wait for complete to be called.
  await Promise.resolve();

  assert_false(promiseResolved,
      "Promise should not resolve until after the source completes");

  source.complete();
  assert_array_equals(logs, [
    "subscribed to source",
    "Predicate called with a, 0",
    "Predicate called with b, 1",
    "teardown",
  ], "Teardown function called immediately after the source completes");

  const result = await resultPromise;

  assert_true(result,
      "Promise resolves with true if all values pass the predicate");
}, "every(): Lifecycle checks when all values pass the predicate");

promise_test(async () => {
  const logs = [];

  const source = createTestSubject({
    onSubscribe: () => logs.push("subscribed to source"),
    onTeardown: () => logs.push("teardown"),
  });

  const resultPromise = source.every((value, index) => {
    logs.push(`Predicate called with ${value}, ${index}`);
    return value === "good";
  });

  let promiseResolved = false;

  resultPromise.then(() => promiseResolved = true);

  assert_array_equals(logs, ["subscribed to source"],
      "calling every() subscribes to the source immediately");

  source.next("good");
  source.next("good");
  assert_array_equals(logs, [
    "subscribed to source",
    "Predicate called with good, 0",
    "Predicate called with good, 1",
  ], "Predicate called with the value and the index");

  assert_false(promiseResolved, "Promise should not resolve until after the predicate fails");

  source.next("bad");
  assert_array_equals(logs, [
    "subscribed to source",
    "Predicate called with good, 0",
    "Predicate called with good, 1",
    "Predicate called with bad, 2",
    "teardown",
  ], "Predicate called with the value and the index, failing predicate immediately aborts subscription to source");

  const result = await resultPromise;

  assert_false(result, "Promise resolves with false if any value fails the predicate");
}, "every(): Lifecycle checks when any value fails the predicate");

promise_test(async () => {
  const source = new Observable(subscriber => {
    subscriber.complete();
  });

  const result = await source.every(() => true);

  assert_true(result,
      "Promise resolves with true if the observable completes without " +
      "emitting a value");
}, "every(): Resolves with true if the observable completes without " +
   "emitting a value");

promise_test(async t => {
  const error = new Error("error from source");
  const source = new Observable(subscriber => {
    subscriber.error(error);
  });

  promise_rejects_exactly(t, error, source.every(() => true),
      "Promise rejects with the error emitted from the source observable");
}, "every(): Rejects with any error emitted from the source observable");

promise_test(async t => {
  const source = new Observable(subscriber => {
    subscriber.next(1);
    subscriber.next(2);
    subscriber.next(3);
    subscriber.complete();
  });

  const error = new Error("bad value");
  const promise = source.every(value => {
    if (value <= 2) return true;
    throw error;
  });

  promise_rejects_exactly(t, error, promise, "Promise rejects with the " +
      "error thrown from the predicate");
}, "every(): Rejects with any error thrown from the predicate");

promise_test(async () => {
  const indices = [];

  const source = new Observable(subscriber => {
    subscriber.next("a");
    subscriber.next("b");
    subscriber.next("c");
    subscriber.complete();
  });

  const value = await source.every((value, index) => {
    indices.push(index);
    return true;
  });

  assert_array_equals(indices, [0, 1, 2]);

  assert_true(value,
      "Promise resolves with true if all values pass the predicate");
}, "every(): Index is passed into the predicate");

promise_test(async t => {
  const source = new Observable(subscriber => {});

  const controller = new AbortController();
  const promise = source.every(() => true, { signal: controller.signal });
  controller.abort();

  promise_rejects_dom(t, 'AbortError', promise, "Promise rejects with a " +
      "DOMException if the source Observable is aborted");
}, "every(): Rejects with a DOMException if the source Observable is aborted");

function createTestSubject(options) {
  const onTeardown = options?.onTeardown;

  const subscribers = new Set();
  const subject = new Observable(subscriber => {
    options?.onSubscribe?.();
    subscribers.add(subscriber);
    subscriber.addTeardown(() => subscribers.delete(subscriber));
    if (onTeardown) {
      subscriber.addTeardown(onTeardown);
    }
  });

  subject.next = (value) => {
    for (const subscriber of Array.from(subscribers)) {
      subscriber.next(value);
    }
  };
  subject.error = (error) => {
    for (const subscriber of Array.from(subscribers)) {
      subscriber.error(error);
    }
  };
  subject.complete = () => {
    for (const subscriber of Array.from(subscribers)) {
      subscriber.complete();
    }
  };
  subject.subscriberCount = () => {
    return subscribers.size;
  };

  return subject;
}
