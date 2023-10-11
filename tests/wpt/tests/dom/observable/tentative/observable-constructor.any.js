test(() => {
  assert_implements(self.Observable, "The Observable interface is not implemented");

  assert_true(
    typeof Observable === "function",
    "Observable constructor is defined"
  );

  assert_throws_js(TypeError, () => { new Observable(); });
}, "Observable constructor");

test(() => {
  assert_implements(self.Subscriber, "The Subscriber interface is not implemented");
  assert_true(
    typeof Subscriber === "function",
    "Subscriber interface is defined as a function"
  );

  assert_throws_js(TypeError, () => { new Subscriber(); });

  new Observable(subscriber => {
    assert_not_equals(subscriber, undefined, "A Subscriber must be passed into the subscribe callback");
    assert_implements(subscriber.next, "A Subscriber object must have a next() method");
    assert_implements(subscriber.complete, "A Subscriber object must have a complete() method");
    assert_implements(subscriber.error, "A Subscriber object must have an error() method");
  }).subscribe();
}, "Subscriber interface is not constructible");

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
    next: () => assert_unreached("next should not be called"),
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
  const error = new Error("error");
  const results = [];

  const source = new Observable((subscriber) => {
    subscriber.next(1);
    throw error;
    // TODO(https://github.com/WICG/observable/issues/76): If we add the
    // `subscriber.closed` attribute, consider a try-finally block to assert
    // that `subscriber.closed` is true after throwing. Also TODO: ensure that
    // that would even be the right behavior.
  });

  source.subscribe({
    next: (x) => results.push(x),
    error: (e) => results.push(e),
    complete: () => assert_unreached("complete should not be called"),
  });

  assert_array_equals(
    results,
    [1, error],
    "should emit values and the throw error synchronously"
  );
}, "Observable should error if initializer throws");

// TODO(https://github.com/WICG/observable/issues/76): If we decide the
// `subscriber.closed` attribute is needed, re-visit these two tests that were
// originally included:
// https://github.com/web-platform-tests/wpt/blob/0246526ca46ef4e5eae8b8e4a87dd905c40f5326/dom/observable/tentative/observable-ctor.any.js#L123-L137.

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
    // TODO(https://github.com/WICG/observable/issues/76): Assert
    // `subscriber.closed` is true, if we add that attribute.
    // assert_true(subscriber.closed, "subscriber is closed after error");
    subscriber.next();
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
  const error = new Error("error");
  let errorReported = false;

  self.addEventListener(
    "error",
    (e) => {
      assert_equals(e.message, "Uncaught (in observable) error");
      assert_equals(e.filename, location.href);
      assert_greater_than(e.lineno, 0);
      assert_greater_than(e.colno, 0);
      assert_equals(e.error, error);
      errorReported = true;
    },
    { once: true }
  );

  const source = new Observable((subscriber) => {
    subscriber.error(error);
  });

  // No error handler provided.
  source.subscribe({
    next: () => assert_unreached("next should not be called"),
    complete: () => assert_unreached("complete should not be called"),
  });

  assert_true(errorReported);
}, "Errors pushed to the subscriber that are not handled by the subscription " +
   "are reported to the global");

test(() => {
  const error = new Error("error");
  let errorReported = false;

  self.addEventListener(
    "error",
    (e) => {
      assert_equals(e.message, "Uncaught (in observable) error");
      assert_equals(e.filename, location.href);
      assert_greater_than(e.lineno, 0);
      assert_greater_than(e.colno, 0);
      assert_equals(e.error, error);
      errorReported = true;
    },
    { once: true }
  );

  const source = new Observable((subscriber) => {
    throw error;
  });

  // No error handler provided.
  source.subscribe({
    next: () => assert_unreached("next should not be called"),
    complete: () => assert_unreached("complete should not be called"),
  });

  assert_true(errorReported);
}, "Errors thrown in the initializer that are not handled by the " +
   "subscription are reported to the global");

test(() => {
  const error = new Error("error");
  const results = [];
  let errorReported = false;

  self.addEventListener(
    "error",
    (e) => {
      assert_equals(e.message, "Uncaught (in observable) error");
      assert_equals(e.filename, location.href);
      assert_greater_than(e.lineno, 0);
      assert_greater_than(e.colno, 0);
      assert_equals(e.error, error);
      errorReported = true;
    },
    { once: true }
  );

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

  assert_true(errorReported);
}, "Subscription reports errors that are pushed after subscriber is closed " +
   "by completion");

test(() => {
  const error = new Error("error");
  const results = [];
  let errorReported = false;

  self.addEventListener(
    "error",
    (e) => {
      assert_equals(e.message, "Uncaught (in observable) error");
      assert_equals(e.filename, location.href);
      assert_greater_than(e.lineno, 0);
      assert_greater_than(e.colno, 0);
      assert_equals(e.error, error);
      errorReported = true;
    },
    { once: true }
  );

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

  assert_array_equals(
    results,
    [1, 2, "complete"],
    "should emit values synchronously, but not error after complete"
  );

  assert_true(errorReported);
}, "Errors thrown by initializer function after subscriber is closed by " +
   "completion are reported");
