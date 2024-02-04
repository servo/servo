// Because we test that the global error handler is called at various times.
setup({allow_uncaught_exception: true});

test(() => {
  assert_implements(self.Observable, "The Observable interface is not implemented");

  assert_true(
    typeof Observable === "function",
    "Observable constructor is defined"
  );

  assert_throws_js(TypeError, () => { new Observable(); });
}, "Observable constructor");

test(() => {
  let initializerCalled = false;
  const source = new Observable(() => {
    initializerCalled = true;
  });

  assert_false(
    initializerCalled,
    "initializer should not be called by construction"
  );
  source.subscribe();
  assert_true(initializerCalled, "initializer should be called by subscribe");
}, "subscribe() can be called with no arguments");

test(() => {
  assert_implements(self.Subscriber, "The Subscriber interface is not implemented");
  assert_true(
    typeof Subscriber === "function",
    "Subscriber interface is defined as a function"
  );

  assert_throws_js(TypeError, () => { new Subscriber(); });

  let initializerCalled = false;
  new Observable(subscriber => {
    assert_not_equals(subscriber, undefined, "A Subscriber must be passed into the subscribe callback");
    assert_implements(subscriber.next, "A Subscriber object must have a next() method");
    assert_implements(subscriber.complete, "A Subscriber object must have a complete() method");
    assert_implements(subscriber.error, "A Subscriber object must have an error() method");
    initializerCalled = true;
  }).subscribe();
  assert_true(initializerCalled, "initializer should be called by subscribe");
}, "Subscriber interface is not constructible");

test(() => {
  let initializerCalled = false;
  const results = [];

  const source = new Observable((subscriber) => {
    initializerCalled = true;
    subscriber.next(1);
    subscriber.next(2);
    subscriber.next(3);
  });

  assert_false(
    initializerCalled,
    "initializer should not be called by construction"
  );

  source.subscribe(x => results.push(x));

  assert_true(initializerCalled, "initializer should be called by subscribe");
  assert_array_equals(
    results,
    [1, 2, 3],
    "should emit values synchronously, but not complete"
  );
}, "Subscribe with just a function as the next handler");

test(() => {
  let initializerCalled = false;
  const results = [];

  const source = new Observable((subscriber) => {
    initializerCalled = true;
    subscriber.next(1);
    subscriber.next(2);
    subscriber.next(3);
    subscriber.complete();
  });

  assert_false(
    initializerCalled,
    "initializer should not be called by construction"
  );

  source.subscribe({
    next: (x) => results.push(x),
    error: () => assert_unreached("error should not be called"),
    complete: () => results.push("complete"),
  });

  assert_true(initializerCalled, "initializer should be called by subscribe");
  assert_array_equals(
    results,
    [1, 2, 3, "complete"],
    "should emit values synchronously"
  );
}, "Observable constructor calls initializer on subscribe");

test(() => {
  const error = new Error("error");
  const results = [];

  const source = new Observable((subscriber) => {
    subscriber.next(1);
    subscriber.next(2);
    subscriber.error(error);
  });

  source.subscribe({
    next: (x) => results.push(x),
    error: (e) => results.push(e),
    complete: () => assert_unreached("complete should not be called"),
  });

  assert_array_equals(
    results,
    [1, 2, error],
    "should emit error synchronously"
  );
}, "Observable error path called synchronously");

test(() => {
  let subscriber;
  new Observable(s => { subscriber = s }).subscribe();
  const {next, complete, error} = subscriber;
  assert_throws_js(TypeError, () => next(1));
  assert_throws_js(TypeError, () => complete());
  assert_throws_js(TypeError, () => error(1));
}, "Subscriber must have receiver");

test(() => {
  let subscriber;
  new Observable(s => { subscriber = s }).subscribe();
  assert_throws_js(TypeError, () => subscriber.next());
  assert_throws_js(TypeError, () => subscriber.error());
}, "Subscriber next & error must recieve argument");

test(() => {
  let subscriber;
  new Observable(s => { subscriber = s }).subscribe();
  assert_true(subscriber.active);
  assert_false(subscriber.signal.aborted);
  subscriber.complete();
  assert_false(subscriber.active);
  assert_true(subscriber.signal.aborted);
}, "Subscriber complete() will set active to false, and abort signal");

test(() => {
  let subscriber;
  new Observable(s => { subscriber = s }).subscribe();
  assert_true(subscriber.active);
  subscriber.active = false;
  assert_true(subscriber.active);
}, "Subscriber active is readonly");

test(() => {
  let subscriber;
  new Observable(s => { subscriber = s }).subscribe();
  assert_false(subscriber.signal.aborted);
  const oldSignal = subscriber.signal;
  const newSignal = AbortSignal.abort();
  subscriber.signal = newSignal;
  assert_false(subscriber.signal.aborted);
  assert_equals(subscriber.signal, oldSignal, "signal did not change");
}, "Subscriber signal is readonly");

test(() => {
  const error = new Error("error");
  const results = [];
  let errorReported = null;
  let innerSubscriber = null;
  let subscriptionActivityInFinallyAfterThrow;
  let subscriptionActivityInErrorHandlerAfterThrow;

  self.addEventListener("error", e => errorReported = e, {once: true});

  const source = new Observable((subscriber) => {
    innerSubscriber = subscriber;
    subscriber.next(1);
    try {
      throw error;
    } finally {
      subscriptionActivityInFinallyAfterThrow = subscriber.active;
    }
  });

  source.subscribe({
    next: (x) => results.push(x),
    error: (e) => {
      subscriptionActivityInErrorHandlerAfterThrow = innerSubscriber.active;
      results.push(e);
    },
    complete: () => assert_unreached("complete should not be called"),
  });

  assert_equals(errorReported, null, "The global error handler should not be " +
      "invoked when the subscribe callback throws an error and the " +
      "subscriber has given an error handler");
  assert_true(subscriptionActivityInFinallyAfterThrow, "Subscriber is " +
      "considered active in finally block before error handler is invoked");
  assert_false(subscriptionActivityInErrorHandlerAfterThrow, "Subscriber is " +
      "considered inactive in error handler block after thrown error");
  assert_array_equals(
    results,
    [1, error],
    "should emit values and the thrown error synchronously"
  );
}, "Observable should error if initializer throws");

test(t => {
  let innerSubscriber = null;
  let activeBeforeComplete = false;
  let activeAfterComplete = false;
  let activeDuringComplete = false;
  let abortedBeforeComplete = false;
  let abortedDuringComplete = false;
  let abortedAfterComplete = false;

  const source = new Observable((subscriber) => {
    innerSubscriber = subscriber;
    activeBeforeComplete = subscriber.active;
    abortedBeforeComplete = subscriber.signal.aborted;

    subscriber.complete();
    activeAfterComplete = subscriber.active;
    abortedAfterComplete = subscriber.signal.aborted;
  });

  source.subscribe({
    complete: () => {
      activeDuringComplete = innerSubscriber.active
      abortedDuringComplete = innerSubscriber.active
    }
  });
  assert_true(activeBeforeComplete, "Subscription is active before complete");
  assert_false(abortedBeforeComplete, "Subscription is not aborted before complete");
  assert_false(activeDuringComplete, "Subscription is not active during complete");
  assert_false(abortedDuringComplete, "Subscription is not aborted during complete");
  assert_false(activeAfterComplete, "Subscription is not active after complete");
  assert_true(abortedAfterComplete, "Subscription is aborted after complete");
}, "Subscription is inactive after complete()");

test(t => {
  let innerSubscriber = null;
  let activeBeforeError = false;
  let activeAfterError = false;
  let activeDuringError = false;
  let abortedBeforeError = false;
  let abortedDuringError = false;
  let abortedAfterError = false;

  const error = new Error("error");
  const source = new Observable((subscriber) => {
    innerSubscriber = subscriber;
    activeBeforeError = subscriber.active;
    abortedBeforeError = subscriber.signal.aborted;

    subscriber.error(error);
    activeAfterError = subscriber.active;
    abortedAfterError = subscriber.signal.aborted;
  });

  source.subscribe({
    error: () => {
      activeDuringError = innerSubscriber.active
    }
  });
  assert_true(activeBeforeError, "Subscription is active before error");
  assert_false(abortedBeforeError, "Subscription is not aborted before error");
  assert_false(activeDuringError, "Subscription is not active during error");
  assert_false(abortedDuringError, "Subscription is not aborted during error");
  assert_false(activeAfterError, "Subscription is not active after error");
  assert_true(abortedAfterError, "Subscription is not aborted after error");
}, "Subscription is inactive after error()");

test(t => {
  let innerSubscriber;
  let initialActivity;
  let initialSignalAborted;

  const source = new Observable((subscriber) => {
    innerSubscriber = subscriber;
    initialActivity = subscriber.active;
    initialSignalAborted = subscriber.signal.aborted;
  });

  source.subscribe({}, {signal: AbortSignal.abort('Initially aborted')});
  assert_false(initialActivity);
  assert_true(initialSignalAborted);
  assert_equals(innerSubscriber.signal.reason, 'Initially aborted');
}, "Subscription is inactive when aborted signal is passed in");

test(() => {
  let outerSubscriber = null;

  const source = new Observable(subscriber => outerSubscriber = subscriber);

  const controller = new AbortController();
  source.subscribe({}, {signal: controller.signal});

  assert_not_equals(controller.signal, outerSubscriber.signal);
}, "Subscriber#signal is not the same AbortSignal as the one passed into `subscribe()`");

test(() => {
  const results = [];

  const source = new Observable((subscriber) => {
    subscriber.next(1);
    subscriber.next(2);
    subscriber.complete();
    subscriber.next(3);
  });

  source.subscribe({
    next: (x) => results.push(x),
    error: () => assert_unreached("error should not be called"),
    complete: () => results.push("complete"),
  });

  assert_array_equals(
    results,
    [1, 2, "complete"],
    "should emit values synchronously, but not nexted values after complete"
  );
}, "Subscription does not emit values after completion");

test(() => {
  const error = new Error("error");
  const results = [];

  const source = new Observable((subscriber) => {
    subscriber.next(1);
    subscriber.next(2);
    subscriber.error(error);
    subscriber.next(3);
  });

  source.subscribe({
    next: (x) => results.push(x),
    error: (e) => results.push(e),
    complete: () => assert_unreached("complete should not be called"),
  });

  assert_array_equals(
    results,
    [1, 2, error],
    "should emit values synchronously, but not nexted values after error"
  );
}, "Subscription does not emit values after error");

test(() => {
  const error = new Error("error");
  const results = [];

  const source = new Observable((subscriber) => {
    subscriber.next(1);
    subscriber.next(2);
    subscriber.error(error);
    assert_false(subscriber.active, "subscriber is closed after error");
    subscriber.next(3);
    subscriber.complete();
  });

  source.subscribe({
    next: (x) => results.push(x),
    error: (error) => results.push(error),
    complete: () => assert_unreached("complete should not be called"),
  });

  assert_array_equals(results, [1, 2, error], "should emit synchronously");
}, "Completing or nexting a subscriber after an error does nothing");

test(() => {
  const error = new Error("custom error");
  let errorReported = null;

  self.addEventListener("error", e => errorReported = e, { once: true });

  const source = new Observable((subscriber) => {
    subscriber.error(error);
  });

  // No error handler provided...
  source.subscribe({
    next: () => assert_unreached("next should not be called"),
    complete: () => assert_unreached("complete should not be called"),
  });

  // ... still the exception is reported to the global.
  assert_true(errorReported !== null, "Exception was reported to global");
  assert_equals(errorReported.message, "Uncaught Error: custom error", "Error message matches");
  assert_greater_than(errorReported.lineno, 0, "Error lineno is greater than 0");
  assert_greater_than(errorReported.colno, 0, "Error lineno is greater than 0");
  assert_equals(errorReported.error, error, "Error object is equivalent");
}, "Errors pushed to the subscriber that are not handled by the subscription " +
   "are reported to the global");

test(() => {
  const error = new Error("custom error");
  let errorReported = null;

  self.addEventListener("error", e => errorReported = e, { once: true });

  const source = new Observable((subscriber) => {
    throw error;
  });

  // No error handler provided...
  source.subscribe({
    next: () => assert_unreached("next should not be called"),
    complete: () => assert_unreached("complete should not be called"),
  });

  // ... still the exception is reported to the global.
  assert_true(errorReported !== null, "Exception was reported to global");
  assert_equals(errorReported.message, "Uncaught Error: custom error", "Error message matches");
  assert_greater_than(errorReported.lineno, 0, "Error lineno is greater than 0");
  assert_greater_than(errorReported.colno, 0, "Error lineno is greater than 0");
  assert_equals(errorReported.error, error, "Error object is equivalent");
}, "Errors thrown in the initializer that are not handled by the " +
   "subscription are reported to the global");

test(() => {
  const error = new Error("custom error");
  const results = [];
  let errorReported = null;

  self.addEventListener("error", e => errorReported = e, { once: true });

  const source = new Observable((subscriber) => {
    subscriber.next(1);
    subscriber.next(2);
    subscriber.complete();
    subscriber.error(error);
  });

  source.subscribe({
    next: (x) => results.push(x),
    error: () => assert_unreached("error should not be called"),
    complete: () => results.push("complete"),
  });

  assert_array_equals(
    results,
    [1, 2, "complete"],
    "should emit values synchronously, but not error values after complete"
  );

  // Error reporting still happens even after  the subscription is closed.
  assert_true(errorReported !== null, "Exception was reported to global");
  assert_equals(errorReported.message, "Uncaught Error: custom error", "Error message matches");
  assert_greater_than(errorReported.lineno, 0, "Error lineno is greater than 0");
  assert_greater_than(errorReported.colno, 0, "Error lineno is greater than 0");
  assert_equals(errorReported.error, error, "Error object is equivalent");
}, "Subscription reports errors that are pushed after subscriber is closed " +
   "by completion");

test(t => {
  const error = new Error("custom error");
  const results = [];
  let errorReported = null;

  self.addEventListener("error", e => errorReported = e, { once: true });

  const source = new Observable((subscriber) => {
    subscriber.next(1);
    subscriber.next(2);
    subscriber.complete();
    throw error;
  });

  source.subscribe({
    next: (x) => results.push(x),
    error: () => assert_unreached("error should not be called"),
    complete: () => results.push("complete"),
  });

  assert_array_equals(results, [1, 2, "complete"],
    "should emit values synchronously, but not error after complete"
  );

  assert_true(errorReported !== null, "Exception was reported to global");
  assert_true(errorReported.message.includes("custom error"), "Error message matches");
  assert_greater_than(errorReported.lineno, 0, "Error lineno is greater than 0");
  assert_greater_than(errorReported.colno, 0, "Error lineno is greater than 0");
  assert_equals(errorReported.error, error, "Error object is equivalent");
}, "Errors thrown by initializer function after subscriber is closed by " +
   "completion are reported");

test(() => {
  const error1 = new Error("error 1");
  const error2 = new Error("error 2");
  const results = [];
  let errorReported = null;

  self.addEventListener("error", e => errorReported = e, { once: true });

  const source = new Observable((subscriber) => {
    subscriber.next(1);
    subscriber.next(2);
    subscriber.error(error1);
    throw error2;
  });

  source.subscribe({
    next: (x) => results.push(x),
    error: (error) => results.push(error),
    complete: () => assert_unreached("complete should not be called"),
  });

  assert_array_equals(
    results,
    [1, 2, error1],
    "should emit values synchronously, but not nexted values after error"
  );

  assert_true(errorReported !== null, "Exception was reported to global");
  assert_true(errorReported.message.includes("error 2"), "Error message matches");
  assert_greater_than(errorReported.lineno, 0, "Error lineno is greater than 0");
  assert_greater_than(errorReported.colno, 0, "Error lineno is greater than 0");
  assert_equals(errorReported.error, error2, "Error object is equivalent");
}, "Errors thrown by initializer function after subscriber is closed by " +
   "error are reported");

test(() => {
  const error1 = new Error("error 1");
  const error2 = new Error("error 2");
  const results = [];
  let errorReported = null;

  self.addEventListener("error", e => errorReported = e, { once: true });

  const source = new Observable((subscriber) => {
    subscriber.next(1);
    subscriber.next(2);
    subscriber.error(error1);
    subscriber.error(error2);
  });

  source.subscribe({
    next: (x) => results.push(x),
    error: (error) => results.push(error),
    complete: () => assert_unreached("complete should not be called"),
  });

  assert_array_equals(
    results,
    [1, 2, error1],
    "should emit values synchronously, but not nexted values after error"
  );

  assert_true(errorReported !== null, "Exception was reported to global");
  assert_true(errorReported.message.includes("error 2"), "Error message matches");
  assert_greater_than(errorReported.lineno, 0, "Error lineno is greater than 0");
  assert_greater_than(errorReported.colno, 0, "Error lineno is greater than 0");
  assert_equals(errorReported.error, error2, "Error object is equivalent");
}, "Errors pushed by initializer function after subscriber is closed by " +
   "error are reported");

test(() => {
  const results = [];
  const target = new EventTarget();

  const source = new Observable((subscriber) => {
    target.addEventListener('custom event', e => {
      subscriber.next(1);
      subscriber.complete();
      subscriber.error('not a real error');
    });
  });

  source.subscribe({
    next: (x) => results.push(x),
    error: (error) => results.push(error),
    complete: () => {
      results.push('complete'),
      // Re-entrantly tries to invoke `complete()`. However, this function must
      // only ever run once.
      target.dispatchEvent(new Event('custom event'));
    },
  });

  target.dispatchEvent(new Event('custom event'));

  assert_array_equals(
    results,
    [1, 'complete'],
    "complete() can only be called once, and cannot invoke other Observer methods"
  );
}, "Subscriber#complete() cannot re-entrantly invoke itself");

test(() => {
  const results = [];
  const target = new EventTarget();

  const source = new Observable((subscriber) => {
    target.addEventListener('custom event', e => {
      subscriber.next(1);
      subscriber.error('not a real error');
      subscriber.complete();
    });
  });

  source.subscribe({
    next: (x) => results.push(x),
    error: (error) => {
      results.push('error'),
      // Re-entrantly tries to invoke `error()`. However, this function must
      // only ever run once.
      target.dispatchEvent(new Event('custom event'));
    },
    complete: () => results.push('complete'),
  });

  target.dispatchEvent(new Event('custom event'));

  assert_array_equals(
    results,
    [1, 'error'],
    "error() can only be called once, and cannot invoke other Observer methods"
  );
}, "Subscriber#error() cannot re-entrantly invoke itself");

test(() => {
  const results = [];
  let innerSubscriber = null;
  let activeDuringTeardown1 = null;
  let abortedDuringTeardown1 = null;
  let activeDuringTeardown2 = null;
  let abortedDuringTeardown2 = null;

  const source = new Observable((subscriber) => {
    assert_true(subscriber.active);
    assert_false(subscriber.signal.aborted);
    results.push('subscribe() callback');
    innerSubscriber = subscriber;

    subscriber.signal.addEventListener('abort', () => {
      assert_false(subscriber.active);
      assert_true(subscriber.signal.aborted);
      results.push('inner abort handler');
      subscriber.next('next from inner abort handler');
      subscriber.complete();
    });

    subscriber.addTeardown(() => {
      activeDuringTeardown1 = subscriber.active;
      abortedDuringTeardown1 = subscriber.signal.aborted;
      results.push('teardown 1');
    });

    subscriber.addTeardown(() => {
      activeDuringTeardown2 = subscriber.active;
      abortedDuringTeardown2 = subscriber.signal.aborted;
      results.push('teardown 2');
    });
  });

  const ac = new AbortController();
  source.subscribe({
    // This should never get called. If it is, the array assertion below will fail.
    next: (x) => results.push(x),
    complete: () => results.push('complete()')
  }, {signal: ac.signal});

  ac.signal.addEventListener('abort', () => {
    results.push('outer abort handler');
    assert_true(ac.signal.aborted);
    assert_false(innerSubscriber.signal.aborted);
  });

  assert_array_equals(results, ['subscribe() callback']);
  ac.abort();
  results.push('abort() returned');
  assert_array_equals(results, [
      'subscribe() callback',
      'outer abort handler', 'teardown 2', 'teardown 1',
      'inner abort handler', 'abort() returned',
  ]);
  assert_false(activeDuringTeardown1, 'should not be active during teardown callback 1');
  assert_false(activeDuringTeardown2, 'should not be active during teardown callback 2');
  assert_true(abortedDuringTeardown1, 'should be aborted during teardown callback 1');
  assert_true(abortedDuringTeardown2, 'should be aborted during teardown callback 2');
}, "Unsubscription lifecycle");

test(t => {
  const source = new Observable((subscriber) => {
    let n = 0;
    while (!subscriber.signal.aborted) {
      assert_true(subscriber.active);
      subscriber.next(n++);
      if (n > 3) {
        assert_unreached("The subscriber should be closed by now");
      }
    }
    assert_false(subscriber.active);
  });

  const ac = new AbortController();
  const results = [];

  source.subscribe({
    next: (x) => {
      results.push(x);
      if (x === 2) {
        ac.abort();
      }
    },
    error: () => results.push('error'),
    complete: () => results.push('complete')
  }, {signal: ac.signal});

  assert_array_equals(
    results,
    [0, 1, 2],
    "should emit values synchronously before abort"
  );
}, "Aborting a subscription should stop emitting values");

test(() => {
  const error = new Error("custom error");
  let errorReported = null;

  self.addEventListener("error", e => errorReported = e, { once: true });

  const source = new Observable(() => {
    throw error;
  });

  try {
    source.subscribe();
  } catch {
    assert_unreached("subscriber() never throws an error");
  }

  assert_true(errorReported !== null, "Exception was reported to global");
  assert_true(errorReported.message.includes("custom error"), "Error message matches");
  assert_greater_than(errorReported.lineno, 0, "Error lineno is greater than 0");
  assert_greater_than(errorReported.colno, 0, "Error lineno is greater than 0");
  assert_equals(errorReported.error, error, "Error object is equivalent");
}, "Calling subscribe should never throw an error synchronously, initializer throws error");

test(() => {
  const error = new Error("custom error");
  let errorReported = null;

  self.addEventListener("error", e => errorReported = e, { once: true });

  const source = new Observable((subscriber) => {
    subscriber.error(error);
  });

  try {
    source.subscribe();
  } catch {
    assert_unreached("subscriber() never throws an error");
  }

  assert_true(errorReported !== null, "Exception was reported to global");
  assert_true(errorReported.message.includes("custom error"), "Error message matches");
  assert_greater_than(errorReported.lineno, 0, "Error lineno is greater than 0");
  assert_greater_than(errorReported.colno, 0, "Error lineno is greater than 0");
  assert_equals(errorReported.error, error, "Error object is equivalent");
}, "Calling subscribe should never throw an error synchronously, subscriber pushes error");

test(() => {
  let addTeardownCalled = false;
  let activeDuringTeardown;

  const source = new Observable((subscriber) => {
    subscriber.addTeardown(() => {
      addTeardownCalled = true;
      activeDuringTeardown = subscriber.active;
    });
  });

  const ac = new AbortController();
  source.subscribe({}, {signal: ac.signal});

  assert_false(addTeardownCalled, "Teardown is not be called upon subscription");
  ac.abort();
  assert_true(addTeardownCalled, "Teardown is called when subscription is aborted");
  assert_false(activeDuringTeardown, "Teardown observers inactive subscription");
}, "Teardown should be called when subscription is aborted");

test(() => {
  const addTeardownsCalled = [];
  // This is used to snapshot `addTeardownsCalled` from within the subscribe
  // callback, for assertion/comparison later.
  let teardownsSnapshot = [];
  const results = [];

  const source = new Observable((subscriber) => {
    subscriber.addTeardown(() => addTeardownsCalled.push("teardown 1"));
    subscriber.addTeardown(() => addTeardownsCalled.push("teardown 2"));

    subscriber.next(1);
    subscriber.next(2);
    subscriber.next(3);
    subscriber.complete();

    // We don't run the actual `assert_array_equals` here because if it fails,
    // it won't be properly caught. This is because assertion failures throw an
    // error, and in subscriber callback, thrown errors result in
    // `window.onerror` handlers being called, which this test file doesn't
    // record as an error (see the first line of this file).
    teardownsSnapshot = addTeardownsCalled;
  });

  source.subscribe({
    next: (x) => results.push(x),
    error: () => results.push("unreached"),
    complete: () => results.push("complete"),
  });

  assert_array_equals(
    results,
    [1, 2, 3, "complete"],
    "should emit values and complete synchronously"
  );

  assert_array_equals(teardownsSnapshot, addTeardownsCalled);
  assert_array_equals(addTeardownsCalled, ["teardown 2", "teardown 1"],
      "Teardowns called in LIFO order synchronously after complete()");
}, "Teardowns should be called when subscription is closed by completion");

test(() => {
  const addTeardownsCalled = [];
  let teardownsSnapshot = [];
  const error = new Error("error");
  const results = [];

  const source = new Observable((subscriber) => {
    subscriber.addTeardown(() => addTeardownsCalled.push("teardown 1"));
    subscriber.addTeardown(() => addTeardownsCalled.push("teardown 2"));

    subscriber.next(1);
    subscriber.next(2);
    subscriber.next(3);
    subscriber.error(error);

    teardownsSnapshot = addTeardownsCalled;
  });

  source.subscribe({
    next: (x) => results.push(x),
    error: (error) => results.push(error),
    complete: () => assert_unreached("complete should not be called"),
  });

  assert_array_equals(
    results,
    [1, 2, 3, error],
    "should emit values and error synchronously"
  );

  assert_array_equals(teardownsSnapshot, addTeardownsCalled);
  assert_array_equals(addTeardownsCalled, ["teardown 2", "teardown 1"],
      "Teardowns called in LIFO order synchronously after error()");
}, "Teardowns should be called when subscription is closed by subscriber pushing an error");

test(() => {
  const addTeardownsCalled = [];
  const error = new Error("error");
  const results = [];

  const source = new Observable((subscriber) => {
    subscriber.addTeardown(() => addTeardownsCalled.push("teardown 1"));
    subscriber.addTeardown(() => addTeardownsCalled.push("teardown 2"));

    subscriber.next(1);
    subscriber.next(2);
    subscriber.next(3);
    throw error;
  });

  source.subscribe({
    next: (x) => results.push(x),
    error: (error) => results.push(error),
    complete: () => assert_unreached("complete should not be called"),
  });

  assert_array_equals(
    results,
    [1, 2, 3, error],
    "should emit values and error synchronously"
  );

  assert_array_equals(addTeardownsCalled, ["teardown 2", "teardown 1"],
      "Teardowns called in LIFO order synchronously after thrown error");
}, "Teardowns should be called when subscription is closed by subscriber throwing error");

test(() => {
  const addTeardownsCalled = [];
  const results = [];
  let firstTeardownInvokedSynchronously = false;
  let secondTeardownInvokedSynchronously = false;

  const source = new Observable((subscriber) => {
    subscriber.addTeardown(() => addTeardownsCalled.push("teardown 1"));
    if (addTeardownsCalled.length === 1) {
      firstTeardownInvokedSynchronously = true;
    }
    subscriber.addTeardown(() => addTeardownsCalled.push("teardown 2"));
    if (addTeardownsCalled.length === 2) {
      secondTeardownInvokedSynchronously = true;
    }

    subscriber.next(1);
    subscriber.next(2);
    subscriber.next(3);
    subscriber.complete();
  });

  const ac = new AbortController();
  ac.abort();
  source.subscribe({
    next: (x) => results.push(x),
    error: (error) => results.push(error),
    complete: () => results.push('complete')
  }, {signal: ac.signal});

  assert_array_equals(results, []);
  assert_true(firstTeardownInvokedSynchronously, "First teardown callback is invoked during addTeardown()");
  assert_true(secondTeardownInvokedSynchronously, "Second teardown callback is invoked during addTeardown()");
  assert_array_equals(addTeardownsCalled, ["teardown 1", "teardown 2"],
      "Teardowns called synchronously upon addition end up in FIFO order");
}, "Teardowns should be called synchronously during addTeardown() if the subscription is inactive");
