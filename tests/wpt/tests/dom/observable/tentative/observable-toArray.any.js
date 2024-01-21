// Because we test that the global error handler is called at various times.
setup({allow_uncaught_exception: true});

promise_test(async () => {
  const observable = new Observable(subscriber => {
    subscriber.next(1);
    subscriber.next(2);
    subscriber.next(3);
    subscriber.complete();
  });

  const array = await observable.toArray();
  assert_array_equals(array, [1, 2, 3]);
}, "toArray(): basic next/complete");

promise_test(async t => {
  let errorReported = null;
  let innerSubscriber = null;
  self.addEventListener('error', e => errorReported = e, {once: true});

  const error = new Error("custom error");
  const observable = new Observable(subscriber => {
    innerSubscriber = subscriber;
    subscriber.error(error);
  });

  try {
    const array = await observable.toArray();
    assert_unreached("toArray() promise must not resolve");
  } catch (e) {
    assert_equals(e, error);
    assert_equals(errorReported, null);

    // Calls to `error()` after the subscription is closed still report the
    // exception.
    innerSubscriber.error(error);
    assert_not_equals(errorReported, null, "Exception was reported to global");
    assert_true(errorReported.message.includes("custom error"), "Error message matches");
    assert_greater_than(errorReported.lineno, 0, "Error lineno is greater than 0");
    assert_greater_than(errorReported.colno, 0, "Error lineno is greater than 0");
    assert_equals(errorReported.error, error, "Error object is equivalent");
  }
}, "toArray(): first error() rejects promise; subsequent error()s report the exceptions");

promise_test(async t => {
  let errorReported = null;
  let innerSubscriber = null;
  self.addEventListener('error', e => errorReported = e, {once: true});

  const error = new Error("custom error");
  const observable = new Observable(subscriber => {
    innerSubscriber = subscriber;
    subscriber.complete();
  });

  const array = await observable.toArray();
  assert_array_equals(array, []);
  assert_equals(errorReported, null);

  // Calls to `error()` after the subscription is closed still report the
  // exception.
  innerSubscriber.error(error);
  assert_not_equals(errorReported, null, "Exception was reported to global");
  assert_true(errorReported.message.includes("custom error"), "Error message matches");
  assert_greater_than(errorReported.lineno, 0, "Error lineno is greater than 0");
  assert_greater_than(errorReported.colno, 0, "Error lineno is greater than 0");
  assert_equals(errorReported.error, error, "Error object is equivalent");
}, "toArray(): complete() resolves promise; subsequent error()s report the exceptions");

promise_test(async () => {
  // This tracks whether `postSubscriptionPromise` has had its then handler run.
  // This helps us keep track of the timing/ordering of everything. Calling a
  // Promise-returning operator with an aborted signal must *immediately* reject
  // the returned Promise, which means code "awaiting" it should run before any
  // subsequent Promise resolution/rejection handlers are run.
  let postSubscriptionPromiseResolved = false;
  let subscriptionImmediatelyInactive = false;

  const observable = new Observable(subscriber => {
    const inactive = !subscriber.active;
    subscriptionImmediatelyInactive = inactive;
  });

  const rejectedPromise = observable.toArray({signal: AbortSignal.abort()})
  .then(() => {
    assert_unreached("Operator promise must not resolve its abort signal is " +
                     "rejected");
  }, () => {
    // See the documentation above. The rejection handler (i.e., this code) for
    // immediately-aborted operator Promises runs before any later-scheduled
    // Promise resolution/rejections.
    assert_false(postSubscriptionPromiseResolved,
        "Operator promise rejects before later promise");
  });
  const postSubscriptionPromise =
      Promise.resolve().then(() => postSubscriptionPromiseResolved = true);

  await rejectedPromise;
}, "toArray(): Subscribing with an aborted signal returns an immediately " +
   "rejected promise");

promise_test(async () => {
  let postSubscriptionPromiseResolved = false;

  const observable = new Observable(subscriber => {});
  const controller = new AbortController();
  const arrayPromise = observable.toArray({signal: controller.signal})
  .then(() => {
    assert_unreached("Operator promise must not resolve if its abort signal " +
    "is rejected");
  }, () => {
    assert_false(postSubscriptionPromiseResolved,
                 "controller.abort() synchronously rejects the operator " +
                 "Promise");
  });

  // This must synchronously reject `arrayPromise`, scheduling in the next
  // microtask.
  controller.abort();
  Promise.resolve().then(value => postSubscriptionPromiseResolved = true);

  await arrayPromise;
}, "toArray(): Aborting the passed-in signal rejects the returned promise");

// See https://github.com/WICG/observable/issues/96 for discussion about this.
promise_test(async () => {
  const results = [];

  const observable = new Observable(subscriber => {
    results.push(`Subscribed. active: ${subscriber.active}`);

    subscriber.signal.addEventListener('abort', e => {
      results.push("Inner signal abort event");
      Promise.resolve("Inner signal Promise").then(value => results.push(value));
    });

    subscriber.addTeardown(() => {
      results.push("Teardown");
      Promise.resolve("Teardown Promise").then(value => results.push(value));
    });
  });

  const controller = new AbortController();
  controller.signal.addEventListener('abort', e => {
    results.push("Outer signal abort event");
    Promise.resolve("Outer signal Promise").then(value => results.push(value));
  });

  // Subscribe.
  observable.toArray({signal: controller.signal});
  controller.abort();

  assert_array_equals(results, [
    "Subscribed. active: true",
    "Outer signal abort event",
    "Teardown",
    "Inner signal abort event",
  ], "Events and teardowns are fired in the right ordered");

  // Everything microtask above should be queued up by now, so queue one more
  // final microtask that will run after all of the others, wait for it, and the
  // check `results` is right.
  await Promise.resolve();
  assert_array_equals(results, [
    "Subscribed. active: true",
    "Outer signal abort event",
    "Teardown",
    "Inner signal abort event",
    "Outer signal Promise",
    "Teardown Promise",
    "Inner signal Promise",
  ], "Promises resolve in the right order");
}, "Operator Promise abort ordering");

