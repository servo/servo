test(() => {
  const source = new Observable(subscriber => {
    subscriber.next(1);
    subscriber.next(2);
    subscriber.next(3);
    subscriber.complete();
  });

  let projectionCalls = 0;

  const results = [];

  const flattened = source.flatMap(value => {
    projectionCalls++;
    return new Observable((subscriber) => {
      subscriber.next(value * 10);
      subscriber.next(value * 100);
      subscriber.complete();
    });
  });

  assert_true(flattened instanceof Observable, "flatMap() returns an Observable");
  assert_equals(projectionCalls, 0,
      "Projection is not called until subscription starts");

  flattened.subscribe({
    next: v => results.push(v),
    error: () => results.push("error"),
    complete: () => results.push("complete"),
  });

  assert_equals(projectionCalls, 3,
      "Mapper is called three times, once for each source Observable value");
  assert_array_equals(results, [10, 100, 20, 200, 30, 300, "complete"],
      "flatMap() results are correct");
}, "flatMap(): Flattens simple source Observable properly");

test(() => {
  const error = new Error("error");
  const source = new Observable(subscriber => {
    subscriber.next(1);
    subscriber.next(2);
    subscriber.error(error);
    subscriber.next(3);
  });

  const flattened = source.flatMap(value => {
    return new Observable(subscriber => {
      subscriber.next(value * 10);
      subscriber.next(value * 100);
      subscriber.complete();
    });
  });

  const results = [];

  flattened.subscribe({
    next: v => results.push(v),
    error: e => results.push(e),
    complete: () => results.push("complete"),
  });

  assert_array_equals(results, [10, 100, 20, 200, error],
      "Source error is passed through to the flatMap() Observable");
}, "flatMap(): Returned Observable passes through source Observable errors");

test(() => {
  const results = [];
  const error = new Error("error");
  const source = new Observable(subscriber => {
    subscriber.next(1);
    results.push(subscriber.active ? "active" : "inactive");
    subscriber.next(2);
    results.push(subscriber.active ? "active" : "inactive");
    subscriber.next(3);
    subscriber.complete();
  });

  const flattened = source.flatMap((value) => {
    return new Observable((subscriber) => {
      subscriber.next(value * 10);
      subscriber.next(value * 100);
      if (value === 2) {
        subscriber.error(error);
      } else {
        subscriber.complete();
      }
    });
  });

  flattened.subscribe({
    next: v => results.push(v),
    error: e => results.push(e),
    complete: () => results.push("complete"),
  });

  assert_array_equals(results, [10, 100, "active", 20, 200, error, "inactive"],
      "Inner subscription error gets surfaced");
}, "flatMap(): Outer Subscription synchronously becomes inactive when an " +
   "'inner' Observable emits an error");

test(() => {
  const results = [];
  const error = new Error("error");
  const source = new Observable(subscriber => {
    subscriber.next(1);
    subscriber.next(2);
    subscriber.next(3);
    results.push(subscriber.active ? "active" : "inactive");
    subscriber.complete();
  });

  const flattened = source.flatMap(value => {
    if (value === 3) {
      throw error;
    }
    return new Observable(subscriber => {
      subscriber.next(value * 10);
      subscriber.next(value * 100);
      subscriber.complete();
    });
  });

  flattened.subscribe({
    next: v => results.push(v),
    error: e => results.push(e),
    complete: () => results.push("complete"),
  });

  assert_array_equals(results, [10, 100, 20, 200, error, "inactive"],
      "Inner subscriber thrown error gets surfaced");
}, "flatMap(): Outer Subscription synchronously becomes inactive when an " +
   "'inner' Observable throws an error");

test(() => {
  const source = createTestSubject();
  const inner1 = createTestSubject();
  const inner2 = createTestSubject();

  const flattened = source.flatMap(value => {
    if (value === 1) {
      return inner1;
    }

    return inner2;
  });

  const results = [];

  flattened.subscribe({
    next: v => results.push(v),
    error: e => results.push(e),
    complete: () => results.push("complete"),
  });

  assert_array_equals(results, []);

  source.next(1);
  assert_equals(inner1.subscriberCount(), 1, "inner1 gets subscribed to");

  source.next(2);
  assert_equals(inner2.subscriberCount(), 0,
      "inner2 is queued, not subscribed to until inner1 completes");

  assert_array_equals(results, []);

  inner1.next(100);
  inner1.next(101);

  assert_array_equals(results, [100, 101]);

  inner1.complete();
  assert_equals(inner1.subscriberCount(), 0,
      "inner1 becomes inactive once it completes");
  assert_equals(inner2.subscriberCount(), 1,
      "inner2 gets un-queued and subscribed to once inner1 completes");

  inner2.next(200);
  inner2.next(201);
  assert_array_equals(results, [100, 101, 200, 201]);

  inner2.complete();
  assert_equals(inner2.subscriberCount(), 0,
      "inner2 becomes inactive once it completes");
  assert_equals(source.subscriberCount(), 1,
      "source is not unsubscribed from yet, since it has not completed");
  assert_array_equals(results, [100, 101, 200, 201]);

  source.complete();
  assert_equals(source.subscriberCount(), 0,
      "source unsubscribed from after it completes");

  assert_array_equals(results, [100, 101, 200, 201, "complete"]);
}, "flatMap(): result Observable does not complete until source and inner " +
   "Observables all complete");

test(() => {
  const source = createTestSubject();
  const inner1 = createTestSubject();
  const inner2 = createTestSubject();

  const flattened = source.flatMap(value => {
    if (value === 1) {
      return inner1;
    }

    return inner2;
  });

  const results = [];

  flattened.subscribe({
    next: v => results.push(v),
    error: e => results.push(e),
    complete: () => results.push("complete"),
  });

  assert_array_equals(results, []);

  source.next(1);
  source.next(2);
  assert_equals(inner1.subscriberCount(), 1, "inner1 gets subscribed to");
  assert_equals(inner2.subscriberCount(), 0,
      "inner2 is queued, not subscribed to until inner1 completes");

  assert_array_equals(results, []);

  // Before `inner1` pushes any values, we first complete the source Observable.
  // This will not fire completion of the Observable returned from `flatMap()`,
  // because there are two values (corresponding to inner Observables) that are
  // queued to the inner queue that need to be processed first. Once the last
  // one of *those* completes (i.e., `inner2.complete()` further down), then the
  // returned Observable can finally complete.
  source.complete();
  assert_equals(source.subscriberCount(), 0,
      "source becomes inactive once it completes");

  inner1.next(100);
  inner1.next(101);

  assert_array_equals(results, [100, 101]);

  inner1.complete();
  assert_array_equals(results, [100, 101],
      "Outer completion not triggered after inner1 completes");
  assert_equals(inner2.subscriberCount(), 1,
      "inner2 gets un-queued and subscribed after inner1 completes");

  inner2.next(200);
  inner2.next(201);
  assert_array_equals(results, [100, 101, 200, 201]);

  inner2.complete();
  assert_equals(inner2.subscriberCount(), 0,
      "inner2 becomes inactive once it completes");
  assert_array_equals(results, [100, 101, 200, 201, "complete"]);
}, "flatMap(): result Observable does not complete after source Observable " +
   "completes while there are still queued inner Observables to process " +
   "Observables all complete");

test(() => {
  const source = createTestSubject();
  const inner = createTestSubject();
  const result = source.flatMap(() => inner);

  const ac = new AbortController();

  result.subscribe({}, { signal: ac.signal, });

  source.next(1);

  assert_equals(inner.subscriberCount(), 1,
      "inner Observable subscribed to once source emits it");

  ac.abort();

  assert_equals(source.subscriberCount(), 0,
      "source unsubscribed from, once outer signal is aborted");

  assert_equals(inner.subscriberCount(), 0,
      "inner Observable unsubscribed from once the outer Observable is " +
      "subscribed from, as a result of the outer signal being aborted");
}, "flatMap(): source and inner active Observables are both unsubscribed " +
   "from once the outer subscription signal is aborted");

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
