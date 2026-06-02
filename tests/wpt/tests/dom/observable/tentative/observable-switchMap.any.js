test(() => {
  const source = createTestSubject();
  const inner1 = createTestSubject();
  const inner2 = createTestSubject();

  const result = source.switchMap((value, index) => {
    if (value === 1) {
      return inner1;
    }
    if (value === 2) {
      return inner2;
    }
    throw new Error("invalid ");
  });

  const results = [];

  result.subscribe({
    next: v => results.push(v),
    error: e => results.push(e),
    complete: () => results.push("complete"),
  });

  assert_equals(source.subscriberCount(), 1,
      "source observable is subscribed to");

  source.next(1);
  assert_equals(inner1.subscriberCount(), 1,
      "inner1 observable is subscribed to");

  inner1.next("1a");
  assert_array_equals(results, ["1a"]);

  inner1.next("1b");
  assert_array_equals(results, ["1a", "1b"]);

  source.next(2);
  assert_equals(inner1.subscriberCount(), 0,
      "inner1 observable is unsubscribed from");
  assert_equals(inner2.subscriberCount(), 1,
      "inner2 observable is subscribed to");

  inner2.next("2a");
  assert_array_equals(results, ["1a", "1b", "2a"]);

  inner2.next("2b");
  assert_array_equals(results, ["1a", "1b", "2a", "2b"]);

  inner2.complete();
  assert_array_equals(results, ["1a", "1b", "2a", "2b"]);

  source.complete();
  assert_array_equals(results, ["1a", "1b", "2a", "2b", "complete"]);
}, "switchMap(): result subscribes to one inner observable at a time, " +
   "unsubscribing from the previous active one when a new one replaces it");

test(() => {
  const source = createTestSubject();
  const inner = createTestSubject();

  const result = source.switchMap(() => inner);

  const results = [];

  result.subscribe({
    next: v => results.push(v),
    error: e => results.push(e),
    complete: () => results.push("complete"),
  });

  assert_equals(source.subscriberCount(), 1,
      "source observable is subscribed to");
  assert_equals(inner.subscriberCount(), 0,
      "inner observable is not subscribed to");

  source.next(1);
  assert_equals(inner.subscriberCount(), 1,
      "inner observable is subscribed to");

  inner.next("a");
  assert_array_equals(results, ["a"]);

  inner.next("b");
  assert_array_equals(results, ["a", "b"]);

  source.complete();
  assert_array_equals(results, ["a", "b"],
      "Result observable does not complete when source observable completes, " +
      "because inner is still active");

  inner.next("c");
  assert_array_equals(results, ["a", "b", "c"]);

  inner.complete();
  assert_array_equals(results, ["a", "b", "c", "complete"],
      "Result observable completes when inner observable completes, because " +
      "source is already complete");
}, "switchMap(): result does not complete when the source observable " +
   "completes, if the inner observable is still active");

test(() => {
  const source = createTestSubject();

  const e = new Error('thrown from mapper');
  const result = source.switchMap(() => {
    throw e;
  });

  const results = [];

  result.subscribe({
    next: v => results.push(v),
    error: e => results.push(e),
    complete: () => results.push("complete"),
  });

  assert_equals(source.subscriberCount(), 1,
      "source observable is subscribed to");

  source.next(1);
  assert_array_equals(results, [e]);
  assert_equals(source.subscriberCount(), 0,
      "source observable is unsubscribed from");
}, "switchMap(): result emits an error if Mapper callback throws an error");

test(() => {
  const source = createTestSubject();
  const inner = createTestSubject();

  const result = source.switchMap(() => inner);

  const results = [];

  result.subscribe({
    next: v => results.push(v),
    error: e => results.push(e),
    complete: () => results.push("complete"),
  });

  source.next(1);
  inner.next("a");
  assert_array_equals(results, ["a"]);

  const e = new Error('error from source');
  source.error(e);
  assert_array_equals(results, ["a", e],
      "switchMap result emits an error if the source emits an error");
  assert_equals(inner.subscriberCount(), 0,
      "inner observable is unsubscribed from");
  assert_equals(source.subscriberCount(), 0,
      "source observable is unsubscribed from");
}, "switchMap(): result emits an error if the source observable emits an " +
   "error");

test(() => {
  const source = createTestSubject();
  const inner = createTestSubject();

  const result = source.switchMap(() => inner);

  const results = [];

  result.subscribe({
    next: v => results.push(v),
    error: e => results.push(e),
    complete: () => results.push("complete"),
  });

  source.next(1);
  inner.next("a");
  assert_array_equals(results, ["a"]);

  const e = new Error("error from inner");
  inner.error(e);
  assert_array_equals(results, ["a", e],
      "result emits an error if the inner observable emits an error");
  assert_equals(inner.subscriberCount(), 0,
      "inner observable is unsubscribed from");
  assert_equals(source.subscriberCount(), 0,
      "source observable is unsubscribed from");
}, "switchMap(): result emits an error if the inner observable emits an error");

test(() => {
  const results = [];
  const source = new Observable(subscriber => {
    subscriber.next(1);
    subscriber.addTeardown(() => {
      results.push('source teardown');
    });
    subscriber.signal.onabort = e => {
      results.push('source onabort');
    };
  });

  const inner = new Observable(subscriber => {
    subscriber.addTeardown(() => {
      results.push('inner teardown');
    });
    subscriber.signal.onabort = () => {
      results.push('inner onabort');
    };
  });

  const result = source.switchMap(() => inner);

  const ac = new AbortController();
  result.subscribe({
    next: v => results.push(v),
    error: e => results.error(e),
    complete: () => results.complete("complete"),
  }, {signal: ac.signal});

  ac.abort();
  assert_array_equals(results, [
    "source onabort",
    "source teardown",
    "inner onabort",
    "inner teardown",
  ], "Unsubscription order is correct");
}, "switchMap(): should unsubscribe in the correct order when user aborts " +
   "the subscription");

// A helper function to create an Observable that can be externally controlled
// and examined for testing purposes.
function createTestSubject() {
  const subscribers = new Set();
  const subject = new Observable(subscriber => {
    subscribers.add(subscriber);
    subscriber.addTeardown(() => subscribers.delete(subscriber));
  });

  subject.next = value => {
    for (const subscriber of Array.from(subscribers)) {
      subscriber.next(value);
    }
  };
  subject.error = error => {
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
