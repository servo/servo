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
  const observable = target.on('custom');
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

// The result of the @@iterator method of the converted object is called:
//   1. Once on conversion (to test that the value is an iterable).
//   2. Once on subscription, to re-pull the iterator implementation from the
//      raw JS object that the Observable owns once synchronous iteration is
//      about to begin.
test(() => {
  let numTimesSymbolIteratorCalled = 0;
  let numTimesNextCalled = 0;

  const iterable = {
    [Symbol.iterator]() {
      numTimesSymbolIteratorCalled++;
      return {
        next() {
          numTimesNextCalled++;
          return {value: undefined, done: true};
        }
      };
    }
  };

  const observable = Observable.from(iterable);

  assert_equals(numTimesSymbolIteratorCalled, 1,
      "Observable.from(iterable) invokes the @@iterator method getter once");
  assert_equals(numTimesNextCalled, 0,
      "Iterator next() is not called until subscription");

  // Override iterable's `[Symbol.iterator]` protocol with an error-throwing
  // function. We assert that on subscription, this method (the new `@@iterator`
  // implementation), is called because only the raw JS object gets stored in
  // the Observable that results in conversion. This raw value must get
  // re-converted to an iterable once iteration is about to start.
  const customError = new Error('@@iterator override error');
  iterable[Symbol.iterator] = () => {
    throw customError;
  };

  let thrownError = null;
  observable.subscribe({
    error: e => thrownError = e,
  });

  assert_equals(thrownError, customError,
      "Error thrown from next() is passed to the error() handler");

  assert_equals(numTimesSymbolIteratorCalled, 1,
      "Subscription re-invokes @@iterator method, which now is a different " +
      "method that does *not* increment our assertion value");
  assert_equals(numTimesNextCalled, 0, "Iterator next() is never called");
}, "from(): [Symbol.iterator] side-effects (one observable)");

// Similar to the above test, but with more Observables!
test(() => {
  let numTimesSymbolIteratorCalled = 0;
  let numTimesNextCalled = 0;

  const iterable = {
    [Symbol.iterator]() {
      numTimesSymbolIteratorCalled++;
      return {
        next() {
          numTimesNextCalled++;
          return {value: undefined, done: true};
        }
      };
    }
  };

  const obs1 = Observable.from(iterable);
  const obs2 = Observable.from(iterable);
  const obs3 = Observable.from(iterable);
  const obs4 = Observable.from(obs3);

  assert_equals(numTimesSymbolIteratorCalled, 3, "Observable.from(iterable) invokes the iterator method getter once");
  assert_equals(numTimesNextCalled, 0, "Iterator next() is not called until subscription");

  iterable[Symbol.iterator] = () => {
    throw new Error('Symbol.iterator override error');
  };

  let errorCount = 0;

  const observer = {error: e => errorCount++};
  obs1.subscribe(observer);
  obs2.subscribe(observer);
  obs3.subscribe(observer);
  obs4.subscribe(observer);
  assert_equals(errorCount, 4,
      "Error-throwing `@@iterator` implementation is called once per " +
      "subscription");

  assert_equals(numTimesSymbolIteratorCalled, 3,
      "Subscription re-invokes the iterator method getter once");
  assert_equals(numTimesNextCalled, 0, "Iterator next() is never called");
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

  assert_array_equals(results, ["value", "complete()"], "Observable emits and completes after Promise resolves");
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
