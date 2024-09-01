// Because we test that the global error handler is called at various times.
setup({allow_uncaught_exception: true});

test(() => {
  assert_equals(typeof Observable.from, "function",
      "Observable.from() is a function");
}, "from(): Observable.from() is a function");

test(() => {
  assert_throws_js(TypeError, () => Observable.from(10),
      "Number cannot convert to an Observable");
  assert_throws_js(TypeError, () => Observable.from(true),
      "Boolean cannot convert to an Observable");
  assert_throws_js(TypeError, () => Observable.from("String"),
      "String cannot convert to an Observable");
  assert_throws_js(TypeError, () => Observable.from({a: 10}),
      "Object cannot convert to an Observable");
  assert_throws_js(TypeError, () => Observable.from(Symbol.iterator),
      "Bare Symbol.iterator cannot convert to an Observable");
  assert_throws_js(TypeError, () => Observable.from(Promise),
      "Promise constructor cannot convert to an Observable");
}, "from(): Failed conversions");

test(() => {
  const target = new EventTarget();
  const observable = target.when('custom');
  const from_observable = Observable.from(observable);
  assert_equals(observable, from_observable);
}, "from(): Given an observable, it returns that exact observable");

test(() => {
  let completeCalled = false;
  const results = [];
  const array = [1, 2, 3, 'a', new Date(), 15, [12]];
  const observable = Observable.from(array);
  observable.subscribe({
    next: v => results.push(v),
    error: e => assert_unreached('error is not called'),
    complete: () => completeCalled = true
  });

  assert_array_equals(results, array);
  assert_true(completeCalled);
}, "from(): Given an array");

test(() => {
  const iterable = {
    [Symbol.iterator]() {
      let n = 0;
      return {
        next() {
          n++;
          if (n <= 3) {
            return { value: n, done: false };
          }
          return { value: undefined, done: true };
        },
      };
    },
  };

  const observable = Observable.from(iterable);

  assert_true(observable instanceof Observable, "Observable.from() returns an Observable");

  const results = [];

  observable.subscribe({
    next: (value) => results.push(value),
    error: () => assert_unreached("should not error"),
    complete: () => results.push("complete"),
  });

  assert_array_equals(results, [1, 2, 3, "complete"],
      "Subscription pushes iterable values out to Observable");

  // A second subscription should restart iteration.
  observable.subscribe({
    next: (value) => results.push(value),
    error: () => assert_unreached("should not error"),
    complete: () => results.push("complete2"),
  });

  assert_array_equals(results, [1, 2, 3, "complete", 1, 2, 3, "complete2"],
      "Subscribing again causes another fresh iteration on an un-exhausted iterable");
}, "from(): Iterable converts to Observable");

// This test, and the variants below it, test the web-observable side-effects of
// converting an iterable object to an Observable. Specifically, it tracks
// exactly when the %Symbol.iterator% method is *retrieved* from the object,
// invoked, and what its error-throwing side-effects are.
//
// Even more specifically, we assert that the %Symbol.iterator% method is
// retrieved a single time when converting to an Observable, and then again when
// subscribing to the converted Observable. This makes it possible for the
// %Symbol.iterator% method getter to change return values in between conversion
// and subscription. See https://github.com/WICG/observable/issues/127 for
// related discussion.
test(() => {
  const results = [];

  const iterable = {
    get [Symbol.iterator]() {
      results.push("[Symbol.iterator] method GETTER");
      return function() {
        results.push("[Symbol.iterator implementation]");
        return {
          get next() {
            results.push("next() method GETTER");
            return function() {
              results.push("next() implementation");
              return {value: undefined, done: true};
            };
          },
        };
      };
    },
  };

  const observable = Observable.from(iterable);
  assert_array_equals(results, ["[Symbol.iterator] method GETTER"]);

  let thrownError = null;
  observable.subscribe();
  assert_array_equals(results, [
    "[Symbol.iterator] method GETTER",
    "[Symbol.iterator] method GETTER",
    "[Symbol.iterator implementation]",
    "next() method GETTER",
    "next() implementation"
  ]);
}, "from(): [Symbol.iterator] side-effects (one observable)");

// This tests that once `Observable.from()` detects a non-null and non-undefined
// `[Symbol.iterator]` property, we've committed to converting as an iterable.
// If the value of that property is not callable, we don't silently move on to
// the next conversion type — we throw a TypeError;
test(() => {
  let results = [];
  const iterable = {
    [Symbol.iterator]: 10,
  };

  let errorThrown = null;
  try {
    Observable.from(iterable);
  } catch(e) {
    errorThrown = e;
  }

  assert_true(errorThrown instanceof TypeError);
  assert_equals(errorThrown.message,
      "Failed to execute 'from' on 'Observable': @@iterator must be a " +
      "callable.");
}, "from(): [Symbol.iterator] not callable");

test(() => {
  let results = [];
  const customError = new Error("@@iterator override error");

  const iterable = {
    numTimesCalled: 0,

    // The first time this getter is called, it returns a legitimate function
    // that, when called, returns an iterator. Every other time it returns an
    // error-throwing function that does not return an iterator.
    get [Symbol.iterator]() {
      this.numTimesCalled++;
      results.push("[Symbol.iterator] method GETTER");

      if (this.numTimesCalled === 1) {
        return this.validIteratorImplementation;
      } else {
        return this.errorThrowingIteratorImplementation;
      }
    },

    validIteratorImplementation: function() {
      results.push("[Symbol.iterator implementation]");
      return {
        get next() {
          results.push("next() method GETTER");
          return function() {
            results.push("next() implementation");
            return {value: undefined, done: true};
          }
        }
      };
    },
    errorThrowingIteratorImplementation: function() {
      results.push("Error-throwing [Symbol.iterator] implementation");
      throw customError;
    },
  };

  const observable = Observable.from(iterable);
  assert_array_equals(results, [
    "[Symbol.iterator] method GETTER",
  ]);

  // Override iterable's `[Symbol.iterator]` protocol with an error-throwing
  // function. We assert that on subscription, this method (the new `@@iterator`
  // implementation), is called because only the raw JS object gets stored in
  // the Observable that results in conversion. This raw value must get
  // re-converted to an iterable once iteration is about to start.

  let thrownError = null;
  observable.subscribe({
    error: e => thrownError = e,
  });

  assert_equals(thrownError, customError,
      "Error thrown from next() is passed to the error() handler");
  assert_array_equals(results, [
    // Old:
    "[Symbol.iterator] method GETTER",
    // New:
    "[Symbol.iterator] method GETTER",
    "Error-throwing [Symbol.iterator] implementation"
  ]);
}, "from(): [Symbol.iterator] is not cached");

// Similar to the above test, but with more Observables!
test(() => {
  const results = [];
  let numTimesSymbolIteratorCalled = 0;
  let numTimesNextCalled = 0;

  const iterable = {
    get [Symbol.iterator]() {
      results.push("[Symbol.iterator] method GETTER");
      return this.internalIteratorImplementation;
    },
    set [Symbol.iterator](func) {
      this.internalIteratorImplementation = func;
    },

    internalIteratorImplementation: function() {
      results.push("[Symbol.iterator] implementation");
      return {
        get next() {
          results.push("next() method GETTER");
          return function() {
            results.push("next() implementation");
            return {value: undefined, done: true};
          };
        },
      };
    },
  };

  const obs1 = Observable.from(iterable);
  const obs2 = Observable.from(iterable);
  const obs3 = Observable.from(iterable);
  const obs4 = Observable.from(obs3);
  assert_equals(obs3, obs4);

  assert_array_equals(results, [
    "[Symbol.iterator] method GETTER",
    "[Symbol.iterator] method GETTER",
    "[Symbol.iterator] method GETTER",
  ]);

  obs1.subscribe();
  assert_array_equals(results, [
    // Old:
    "[Symbol.iterator] method GETTER",
    "[Symbol.iterator] method GETTER",
    "[Symbol.iterator] method GETTER",
    // New:
    "[Symbol.iterator] method GETTER",
    "[Symbol.iterator] implementation",
    "next() method GETTER",
    "next() implementation",
  ]);

  iterable[Symbol.iterator] = () => {
    results.push("Error-throwing [Symbol.iterator] implementation");
    throw new Error('Symbol.iterator override error');
  };

  let errorCount = 0;

  const observer = {error: e => errorCount++};
  obs2.subscribe(observer);
  obs3.subscribe(observer);
  obs4.subscribe(observer);
  assert_equals(errorCount, 3,
      "Error-throwing `@@iterator` implementation is called once per " +
      "subscription");

  assert_array_equals(results, [
    // Old:
    "[Symbol.iterator] method GETTER",
    "[Symbol.iterator] method GETTER",
    "[Symbol.iterator] method GETTER",
    "[Symbol.iterator] method GETTER",
    "[Symbol.iterator] implementation",
    "next() method GETTER",
    "next() implementation",
    // New:
    "[Symbol.iterator] method GETTER",
    "Error-throwing [Symbol.iterator] implementation",
    "[Symbol.iterator] method GETTER",
    "Error-throwing [Symbol.iterator] implementation",
    "[Symbol.iterator] method GETTER",
    "Error-throwing [Symbol.iterator] implementation",
  ]);
}, "from(): [Symbol.iterator] side-effects (many observables)");

test(() => {
  const customError = new Error('@@iterator next() error');
  const iterable = {
    [Symbol.iterator]() {
      return {
        next() {
          throw customError;
        }
      };
    }
  };

  let thrownError = null;
  Observable.from(iterable).subscribe({
    error: e => thrownError = e,
  });

  assert_equals(thrownError, customError,
      "Error thrown from next() is passed to the error() handler");
}, "from(): [Symbol.iterator] next() throws error");

promise_test(async () => {
  const promise = Promise.resolve('value');
  const observable = Observable.from(promise);

  assert_true(observable instanceof Observable, "Converts to Observable");

  const results = [];

  observable.subscribe({
    next: (value) => results.push(value),
    error: () => assert_unreached("error() is not called"),
    complete: () => results.push("complete()"),
  });

  assert_array_equals(results, [], "Observable does not emit synchronously");

  await promise;

  assert_array_equals(results, ["value", "complete()"],
      "Observable emits and completes after Promise resolves");
}, "from(): Converts Promise to Observable");

promise_test(async t => {
  let unhandledRejectionHandlerCalled = false;
  const unhandledRejectionHandler = () => {
    unhandledRejectionHandlerCalled = true;
  };

  self.addEventListener("unhandledrejection", unhandledRejectionHandler);
  t.add_cleanup(() => self.removeEventListener("unhandledrejection", unhandledRejectionHandler));

  const promise = Promise.reject("reason");
  const observable = Observable.from(promise);

  assert_true(observable instanceof Observable, "Converts to Observable");

  const results = [];

  observable.subscribe({
    next: (value) => assert_unreached("next() not called"),
    error: (error) => results.push(error),
    complete: () => assert_unreached("complete() not called"),
  });

  assert_array_equals(results, [], "Observable does not emit synchronously");

  let catchBlockEntered = false;
  try {
    await promise;
  } catch {
    catchBlockEntered = true;
  }

  assert_true(catchBlockEntered, "Catch block entered");
  assert_false(unhandledRejectionHandlerCalled, "No unhandledrejection event");
  assert_array_equals(results, ["reason"],
      "Observable emits error() after Promise rejects");
}, "from(): Converts rejected Promise to Observable. No " +
   "`unhandledrejection` event when error is handled by subscription");

promise_test(async t => {
  let unhandledRejectionHandlerCalled = false;
  const unhandledRejectionHandler = () => {
    unhandledRejectionHandlerCalled = true;
  };

  self.addEventListener("unhandledrejection", unhandledRejectionHandler);
  t.add_cleanup(() => self.removeEventListener("unhandledrejection", unhandledRejectionHandler));

  let errorReported = null;
  self.addEventListener("error", e => errorReported = e, { once: true });

  let catchBlockEntered = false;
  try {
    const promise = Promise.reject("custom reason");
    const observable = Observable.from(promise);

    observable.subscribe();
    await promise;
  } catch {
    catchBlockEntered = true;
  }

  assert_true(catchBlockEntered, "Catch block entered");
  assert_false(unhandledRejectionHandlerCalled,
      "No unhandledrejection event, because error got reported to global");
  assert_not_equals(errorReported, null, "Error was reported to the global");

  assert_true(errorReported.message.includes("custom reason"),
      "Error message matches");
  assert_equals(errorReported.lineno, 0, "Error lineno is 0");
  assert_equals(errorReported.colno, 0, "Error lineno is 0");
  assert_equals(errorReported.error, "custom reason",
      "Error object is equivalent");
}, "from(): Rejections not handled by subscription are reported to the " +
   "global, and still not sent as an unhandledrejection event");

test(() => {
  const results = [];
  const observable = new Observable(subscriber => {
    subscriber.next('from Observable');
    subscriber.complete();
  });

  observable[Symbol.iterator] = () => {
    results.push('Symbol.iterator() called');
    return {
      next() {
        return {value: 'from @@iterator', done: true};
      }
    };
  };

  Observable.from(observable).subscribe({
    next: v => results.push(v),
    complete: () => results.push("complete"),
  });

  assert_array_equals(results, ["from Observable", "complete"]);
}, "from(): Observable that implements @@iterator protocol gets converted " +
   "as an Observable, not iterator");

test(() => {
  const results = [];
  const promise = new Promise(resolve => {
    resolve('from Promise');
  });

  promise[Symbol.iterator] = () => {
    let done = false;
    return {
      next() {
        if (!done) {
          done = true;
          return {value: 'from @@iterator', done: false};
        } else {
          return {value: undefined, done: true};
        }
      }
    };
  };

  Observable.from(promise).subscribe({
    next: v => results.push(v),
    complete: () => results.push("complete"),
  });

  assert_array_equals(results, ["from @@iterator", "complete"]);
}, "from(): Promise that implements @@iterator protocol gets converted as " +
   "an iterable, not Promise");

// When the [Symbol.iterator] method on a given object is undefined, we don't
// try to convert the object to an Observable via the iterable protocol. The
// Observable specification *also* does the same thing if the [Symbol.iterator]
// method is *null*. That is, in that case we also skip the conversion via
// iterable protocol, and continue to try and convert the object as another type
// (in this case, a Promise).
promise_test(async () => {
  const promise = new Promise(resolve => resolve('from Promise'));
  assert_equals(promise[Symbol.iterator], undefined);
  promise[Symbol.iterator] = null;
  assert_equals(promise[Symbol.iterator], null);

  const value = await new Promise(resolve => {
    Observable.from(promise).subscribe(value => resolve(value));
  });

  assert_equals(value, 'from Promise');
}, "from(): Promise whose [Symbol.iterator] returns null converts as Promise");

// This is a more sensitive test, which asserts that even just trying to reach
// for the [Symbol.iterator] method on an object whose *getter* for the
// [Symbol.iterator] method throws an error, results in `Observable#from()`
// rethrowing that error.
test(() => {
  const error = new Error('thrown from @@iterator getter');
  const obj = {
    get [Symbol.iterator]() {
      throw error;
    }
  }

  try {
    Observable.from(obj);
    assert_unreached("from() conversion throws");
  } catch(e) {
    assert_equals(e, error);
  }
}, "from(): Rethrows the error when Converting an object whose @@iterator " +
   "method *getter* throws an error");

test(() => {
  const obj = {};
  // Non-undefined & non-null values of the `@@iterator` property are not
  // allowed. Specifically they fail the the `IsCallable()` test, which fails
  // Observable conversion.
  obj[Symbol.iterator] = 10;

  try {
    Observable.from(obj);
    assert_unreached("from() conversion throws");
  } catch(e) {
    assert_true(e instanceof TypeError);
    assert_equals(e.message,
        "Failed to execute 'from' on 'Observable': @@iterator must be a callable.");
  }
}, "from(): Throws 'callable' error when @@iterator property is a " +
   "non-callable primitive");

// TODO(dom@chromium.org): Add another test like the above, but for
// `[Symbol.asyncIterator] = null` falling back to `[Symbol.iterator]`
// conversion.
