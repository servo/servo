// Because we test that the global error handler is called at various times.
setup({allow_uncaught_exception: true});

promise_test(async () => {
  const source = new Observable(subscriber => {
    let i = 0;
    const interval = setInterval(() => {
      if (i < 5) {
        subscriber.next(++i);
      } else {
        subscriber.complete();
        clearInterval(interval);
      }
    }, 0);
  });

  const result = await source.takeUntil(new Observable(() => {})).toArray();
  assert_array_equals(result, [1, 2, 3, 4, 5]);
}, "takeUntil subscribes to source Observable and mirrors it uninterrupted");

promise_test(async () => {
  const source = new Observable(() => {});
  let notifierSubscribedTo = false;
  const notifier = new Observable(() => notifierSubscribedTo = true);

  source.takeUntil(notifier).subscribe();
  assert_true(notifierSubscribedTo);
}, "takeUntil subscribes to notifier");

// This test is important because ordinarily, calling `subscriber.next()` does
// not cancel a subscription associated with `subscriber`. However, for the
// `takeUntil()` operator, the spec responds to `notifier`'s `next()` by
// unsubscribing from `notifier`, which is what this test asserts.
promise_test(async () => {
  const source = new Observable(subscriber => {});

  let notifierSubscriberActiveBeforeNext;
  let notifierSubscriberActiveAfterNext;
  let teardownCalledAfterNext;
  let notifierSignalAbortedAfterNext;
  const notifier = new Observable(subscriber => {
    let teardownCalled;
    subscriber.addTeardown(() => teardownCalled = true);

    // Calling `next()` causes `takeUntil()` to unsubscribe from `notifier`.
    notifierSubscriberActiveBeforeNext = subscriber.active;
    subscriber.next('value');
    notifierSubscriberActiveAfterNext = subscriber.active;
    teardownCalledAfterNext = (teardownCalled === true);
    notifierSignalAbortedAfterNext = subscriber.signal.aborted;
  });

  let nextOrErrorCalled = false;
  let completeCalled = false;
  source.takeUntil(notifier).subscribe({
    next: () => nextOrErrorCalled = true,
    error: () => nextOrErrorCalled = true,
    complete: () => completeCalled = true,
  });
  assert_true(notifierSubscriberActiveBeforeNext);
  assert_false(notifierSubscriberActiveAfterNext);
  assert_true(teardownCalledAfterNext);
  assert_true(notifierSignalAbortedAfterNext);
  assert_false(nextOrErrorCalled);
  assert_true(completeCalled);
}, "takeUntil: notifier next() unsubscribes to notifier");

// This test is identical to the one above, with the exception being that the
// `notifier` calls `subscriber.error()` instead `subscriber.next()`.
promise_test(async () => {
  const source = new Observable(subscriber => {});

  let notifierSubscriberActiveBeforeNext;
  let notifierSubscriberActiveAfterNext;
  let teardownCalledAfterNext;
  let notifierSignalAbortedAfterNext;
  const notifier = new Observable(subscriber => {
    let teardownCalled;
    subscriber.addTeardown(() => teardownCalled = true);

    // Calling `next()` causes `takeUntil()` to unsubscribe from `notifier`.
    notifierSubscriberActiveBeforeNext = subscriber.active;
    subscriber.error('error');
    notifierSubscriberActiveAfterNext = subscriber.active;
    teardownCalledAfterNext = (teardownCalled === true);
    notifierSignalAbortedAfterNext = subscriber.signal.aborted;
  });

  let nextOrErrorCalled = false;
  let completeCalled = false;
  source.takeUntil(notifier).subscribe({
    next: () => nextOrErrorCalled = true,
    error: () => nextOrErrorCalled = true,
    complete: () => completeCalled = true,
  });
  assert_true(notifierSubscriberActiveBeforeNext);
  assert_false(notifierSubscriberActiveAfterNext);
  assert_true(teardownCalledAfterNext);
  assert_true(notifierSignalAbortedAfterNext);
  assert_false(nextOrErrorCalled);
  assert_true(completeCalled);
}, "takeUntil: notifier error() unsubscribes to notifier");

// Test that `notifier` unsubscribes from source Observable.
promise_test(async t => {
  const results = [];

  const source = new Observable(subscriber => {
    results.push('source subscribed');
    subscriber.addTeardown(() => results.push('source teardown'));
    subscriber.signal.addEventListener('abort',
        e => results.push('source signal abort'));
  });

  let notifierTeardownCalled = false;
  const notifier = new Observable(subscriber => {
    results.push('notifier subscribed');
    subscriber.addTeardown(() => {
      results.push('notifier teardown');
      notifierTeardownCalled = true;
    });
    subscriber.signal.addEventListener('abort',
        e => results.push('notifier signal abort'));

    // Asynchronously shut everything down.
    t.step_timeout(() => subscriber.next('value'));
  });

  let nextOrErrorCalled = false;
  let notifierTeardownCalledBeforeCompleteCallback;
  await new Promise(resolve => {
    source.takeUntil(notifier).subscribe({
      next: () => nextOrErrorCalled = true,
      error: () => nextOrErrorCalled = true,
      complete: () => {
        notifierTeardownCalledBeforeCompleteCallback = notifierTeardownCalled;
        resolve();
      },
    });
  });

  // The outer `Observer#complete()` callback is called before any teardowns are
  // invoked.
  assert_false(nextOrErrorCalled);
  // The notifier/source teardowns are not called by the time the outer
  // `Observer#complete()` callback is invoked, but they are all run *after*
  // (i.e., before `notifier`'s `subscriber.next()` returns internally).
  assert_false(notifierTeardownCalledBeforeCompleteCallback);
  assert_true(notifierTeardownCalled);
  assert_array_equals(results, [
    "notifier subscribed",
    "source subscribed",
    "notifier teardown",
    "notifier signal abort",
    "source teardown",
    "source signal abort"
  ]);
}, "takeUntil: notifier next() unsubscribes from notifier & source observable");

// This test is almost identical to the above test, however instead of the
// `notifier` Observable being the thing that causes the unsubscription from
// `notifier` and `source`, it is the outer composite Observable's
// `SubscribeOptions#signal` being aborted that does this.
promise_test(async t => {
  const results = [];
  // This will get populated later with a function that resolves a promise.
  let resolver;

  const source = new Observable(subscriber => {
    results.push('source subscribed');
    subscriber.addTeardown(() => results.push('source teardown'));
    subscriber.signal.addEventListener('abort', e => {
      results.push('source signal abort');
      // This should be the last thing run in the whole teardown sequence. After
      // this, we can resolve the promise that this test is waiting on, via
      // `resolver`. That'll wrap things up and move us on to the assertions.
      resolver();
    });
  });

  const notifier = new Observable(subscriber => {
    results.push('notifier subscribed');
    subscriber.addTeardown(() => {
      results.push('notifier teardown');
    });
    subscriber.signal.addEventListener('abort',
        e => results.push('notifier signal abort'));
  });

  let observerCallbackCalled = false;
  await new Promise(resolve => {
    resolver = resolve;
    const controller = new AbortController();
    source.takeUntil(notifier).subscribe({
      next: () => observerCallbackCalled = true,
      error: () => observerCallbackCalled = true,
      complete: () => observerCallbackCalled = true,
    }, {signal: controller.signal});

    // Asynchronously shut everything down.
    t.step_timeout(() => controller.abort());
  });

  assert_false(observerCallbackCalled);
  assert_array_equals(results, [
    "notifier subscribed",
    "source subscribed",
    "notifier teardown",
    "notifier signal abort",
    "source teardown",
    "source signal abort"
  ]);
}, "takeUntil()'s AbortSignal unsubscribes from notifier & source observable");

promise_test(async () => {
  let sourceSubscribedTo = false;
  const source = new Observable(subscriber => {
    sourceSubscribedTo = true;
  });

  const notifier = new Observable(subscriber => subscriber.next('value'));

  let nextOrErrorCalled = false;
  let completeCalled = false;
  const result = source.takeUntil(notifier).subscribe({
    next: v => nextOrErrorCalled = true,
    error: e => nextOrErrorCalled = true,
    complete: () => completeCalled = true,
  });

  assert_false(sourceSubscribedTo);
  assert_true(completeCalled);
  assert_false(nextOrErrorCalled);
}, "takeUntil: source never subscribed to when notifier synchronously emits a value");

promise_test(async () => {
  let sourceSubscribedTo = false;
  const source = new Observable(subscriber => {
    sourceSubscribedTo = true;
  });

  const notifier = new Observable(subscriber => subscriber.error('error'));

  let nextOrErrorCalled = false;
  let completeCalled = false;
  const result = source.takeUntil(notifier).subscribe({
    next: v => nextOrErrorCalled = true,
    error: e => nextOrErrorCalled = true,
    complete: () => completeCalled = true,
  });

  assert_false(sourceSubscribedTo);
  assert_true(completeCalled);
  assert_false(nextOrErrorCalled);
}, "takeUntil: source never subscribed to when notifier synchronously emits error");

promise_test(async () => {
  const source = new Observable(subscriber => {
    let i = 0;
    const interval = setInterval(() => {
      if (i < 5) {
        subscriber.next(++i);
      } else {
        subscriber.complete();
        clearInterval(interval);
      }
    }, 500);
  });

  const notifier = new Observable(subscriber => subscriber.complete());

  const result = await source.takeUntil(notifier).toArray();
  assert_array_equals(result, [1, 2, 3, 4, 5]);
}, "takeUntil: source is uninterrupted when notifier completes, even synchronously");

promise_test(async () => {
  const results = [];

  let sourceSubscriber;
  let notifierSubscriber;
  const source = new Observable(subscriber => sourceSubscriber = subscriber);
  const notifier = new Observable(subscriber => notifierSubscriber = subscriber);

  source.takeUntil(notifier).subscribe({
    next: v => results.push(v),
    complete: () => results.push("complete"),
  });

  sourceSubscriber.next(1);
  sourceSubscriber.next(2);
  notifierSubscriber.next('notifier value');
  sourceSubscriber.next(3);

  assert_array_equals(results, [1, 2, 'complete']);
}, "takeUntil() mirrors the source Observable until its first next() value");

promise_test(async t => {
  let errorReported = null;

  self.addEventListener("error", e => errorReported = e, { once: true });

  const source = new Observable(() => {});
  const notifier = new Observable(subscriber => {
    t.step_timeout(() => {
      subscriber.error('error 1');
      subscriber.error('error 2');
    });
  });

  let errorCallbackCalled = false;
  await new Promise(resolve => {
    source.takeUntil(notifier).subscribe({
      error: e => errorCallbackCalled = true,
      complete: () => resolve(),
    });
  });

  assert_false(errorCallbackCalled);
  assert_true(errorReported !== null, "Exception was reported to global");
  assert_equals(errorReported.message, "Uncaught error 2", "Error message matches");
  assert_greater_than(errorReported.lineno, 0, "Error lineno is greater than 0");
  assert_greater_than(errorReported.colno, 0, "Error lineno is greater than 0");
  assert_equals(errorReported.error, 'error 2', "Error object is equivalent (just a string)");
}, "takeUntil: notifier calls `Subscriber#error()` twice; second goes to global error handler");
