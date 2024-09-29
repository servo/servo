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

// This test exercises the line of spec prose that says:
//
// "If |asyncIteratorMethodRecord|'s [[Value]] is undefined or null, then jump
// to the step labeled 'From iterable'."
test(() => {
  const sync_iterable = {
    [Symbol.asyncIterator]: null,
    [Symbol.iterator]() {
      return {
        value: 0,
        next() {
          if (this.value === 2)
            return {value: undefined, done: true};
          else
            return {value: this.value++, done: false};
        }
      }
    },
  };

  const results = [];
  const source = Observable.from(sync_iterable).subscribe(v => results.push(v));
  assert_array_equals(results, [0, 1]);
}, "from(): Async iterable protocol null, converts as iterator");

promise_test(async t => {
  const results = [];
  const async_iterable = {
    [Symbol.asyncIterator]() {
      results.push("[Symbol.asyncIterator]() invoked");
      return {
        val: 0,
        next() {
          return new Promise(resolve => {
            t.step_timeout(() => {
              resolve({
                value: this.val,
                done: this.val++ === 4 ? true : false,
              });
            }, 400);
          });
        },
      };
    },
  };

  const source = Observable.from(async_iterable);
  assert_array_equals(results, []);

  await new Promise(resolve => {
    source.subscribe({
      next: v => {
        results.push(`Observing ${v}`);
        queueMicrotask(() => results.push(`next() microtask interleaving (v=${v})`));
      },
      complete: () => {
        results.push('complete()');
        resolve();
      },
    });
  });

  assert_array_equals(results, [
    "[Symbol.asyncIterator]() invoked",
    "Observing 0",
    "next() microtask interleaving (v=0)",
    "Observing 1",
    "next() microtask interleaving (v=1)",
    "Observing 2",
    "next() microtask interleaving (v=2)",
    "Observing 3",
    "next() microtask interleaving (v=3)",
    "complete()",
  ]);
}, "from(): Asynchronous iterable conversion");

// This test is a more chaotic version of the above. It ensures that a single
// Observable can handle multiple in-flight subscriptions to the same underlying
// async iterable without the two subscriptions competing.
//
// This test is added because it is easy to imagine an implementation whereby
// upon subscription, the Observable's internal subscribe callback takes the
// underlying async iterable object, and simply pulls the async iterator off of
// it (by invoking `@@asyncIterator`), and saves it alongside the underlying
// async iterable. This async iterator would be used to manage values as they
// are asynchronously emitted from the underlying object, but this value can get
// OVERWRITTEN by a brand new subscription that comes in before the first
// subscription has completed. In a broken implementation, this overwriting
// would prevent the first subscription from ever completing.
promise_test(async t => {
  const async_iterable = {
    slow: true,
    [Symbol.asyncIterator]() {
      // The first time @@asyncIterator is called, `shouldBeSlow` is true, and
      // when the return object takes closure of it, all values are emitted
      // SLOWLY asynchronously. The second time, `shouldBeSlow` is false, and
      // all values are emitted FAST but still asynchronous.
      const shouldBeSlow = this.slow;
      this.slow = false;

      return {
        val: 0,
        next() {
          // Returns a Promise that resolves in a random amount of time less
          // than a second.
          return new Promise(resolve => {
            t.step_timeout(() => resolve({
              value: `${this.val}-${shouldBeSlow ? 'slow' : 'fast'}`,
              done: this.val++ === 4 ? true : false,
            }), shouldBeSlow ? 200 : 0);
          });
        },
      };
    },
  };

  const results = [];
  const source = Observable.from(async_iterable);

  const subscribeFunction = function(resolve, reject) {
    source.subscribe({
      next: v => results.push(v),
      complete: () => resolve(),
    });

    // A broken implementation will rely on this timeout.
    t.step_timeout(() => reject('TIMEOUT'), 3000);
  }

  const slow_promise = new Promise(subscribeFunction);
  const fast_promise = new Promise(subscribeFunction);
  await Promise.all([slow_promise, fast_promise]);
  assert_array_equals(results, [
    '0-fast',
    '1-fast',
    '2-fast',
    '3-fast',
    '0-slow',
    '1-slow',
    '2-slow',
    '3-slow',
  ]);
}, "from(): Asynchronous iterable multiple in-flight subscriptions competing");
// This test is like the above, ensuring that multiple subscriptions to the same
// sync-iterable-converted-Observable can exist at a time. Since sync iterables
// push all of their values to the Observable synchronously, the way to do this
// is subscribe to the sync iterable Observable *inside* the next handler of the
// same Observable.
test(() => {
  const results = [];

  const array = [1, 2, 3, 4, 5];
  const source = Observable.from(array);
  source.subscribe({
    next: v => {
      results.push(v);
      if (v === 3) {
        // Pushes all 5 values to `results` right after the first instance of `3`.
        source.subscribe({
          next: v => results.push(v),
          complete: () => results.push('inner complete'),
        });
      }
    },
    complete: () => results.push('outer complete'),
  });

  assert_array_equals(results, [
    1, 2, 3,
    1, 2, 3, 4, 5, 'inner complete',
    4, 5, 'outer complete'
  ]);
}, "from(): Sync iterable multiple in-flight subscriptions competing");

promise_test(async () => {
  const async_generator = async function*() {
    yield 1;
    yield 2;
    yield 3;
  };

  const results = [];
  const source = Observable.from(async_generator());

  const subscribeFunction = function(resolve) {
    source.subscribe({
      next: v => results.push(v),
      complete: () => resolve(),
    });
  }
  await new Promise(subscribeFunction);
  assert_array_equals(results, [1, 2, 3]);
  await new Promise(subscribeFunction);
  assert_array_equals(results, [1, 2, 3]);
}, "from(): Asynchronous generator conversion: can only be used once");

// The value returned by an async iterator object's `next()` method is supposed
// to be a Promise. But this requirement "isn't enforced": see [1]. Therefore,
// the Observable spec unconditionally wraps the return value in a resolved
// Promise, as is standard practice [2].
//
// This test ensures that even if the object returned from an async iterator's
// `next()` method is a synchronously-available object with `done: true`
// (instead of a Promise), the `done` property is STILL not retrieved
// synchronously. In other words, we test that the Promise-wrapping is
// implemented.
//
// [1]: https://tc39.es/ecma262/#table-async-iterator-r
// [2]: https://matrixlogs.bakkot.com/WHATWG/2024-08-30#L30
promise_test(async () => {
  const results = [];

  const async_iterable = {
    [Symbol.asyncIterator]() {
      return {
        next() {
          return {
            value: undefined,
            get done() {
              results.push('done() GETTER called');
              return true;
            },
          };
        },
      };
    },
  };

  const source = Observable.from(async_iterable);
  assert_array_equals(results, []);

  queueMicrotask(() => results.push('Microtask queued before subscription'));
  source.subscribe();
  assert_array_equals(results, []);

  await Promise.resolve();
  assert_array_equals(results, [
    "Microtask queued before subscription",
    "done() GETTER called",
  ]);
}, "from(): Promise-wrapping semantics of IteratorResult interface");

// Errors thrown from [Symbol.asyncIterator] are propagated to the observer
// synchronously. This is because in language constructs (i.e., for-await of
// loops) that invoke [Symbol.asyncIterator]() that throw errors, the errors are
// synchronously propagated to script outside of the loop, and are catchable.
// Observables follow this precedent.
test(() => {
  const error = new Error("[Symbol.asyncIterator] error");
  const results = [];
  const async_iterable = {
    [Symbol.asyncIterator]() {
      results.push("[Symbol.asyncIterator]() invoked");
      throw error;
    }
  };

  Observable.from(async_iterable).subscribe({
    error: e => results.push(e),
  });

  assert_array_equals(results, [
    "[Symbol.asyncIterator]() invoked",
    error,
  ]);
}, "from(): Errors thrown in Symbol.asyncIterator() are propagated synchronously");

// AsyncIterable: next() throws exception instead of return Promise. Any errors
// that occur during the the retrieval of `next()` always result in a rejected
// Promise. Therefore, the error makes it to the Observer with microtask timing.
promise_test(async () => {
  const nextError = new Error('next error');
  const async_iterable = {
    [Symbol.asyncIterator]() {
      return {
        get next() {
          throw nextError;
        }
      };
    }
  };

  const results = [];
  Observable.from(async_iterable).subscribe({
    error: e => results.push(e),
  });

  assert_array_equals(results, []);
  // Wait one microtask since the error will be propagated through a rejected
  // Promise managed by the async iterable conversion semantics.
  await Promise.resolve();
  assert_array_equals(results, [nextError]);
}, "from(): Errors thrown in async iterator's next() GETTER are propagated " +
   "in a microtask");
promise_test(async () => {
  const nextError = new Error('next error');
  const async_iterable = {
    [Symbol.asyncIterator]() {
      return {
        next() {
          throw nextError;
        }
      };
    }
  };

  const results = [];
  Observable.from(async_iterable).subscribe({
    error: e => results.push(e),
  });

  assert_array_equals(results, []);
  await Promise.resolve();
  assert_array_equals(results, [nextError]);
}, "from(): Errors thrown in async iterator's next() are propagated in a microtask");

test(() => {
  const results = [];
  const iterable = {
    [Symbol.iterator]() {
      return {
        val: 0,
        next() {
          results.push(`IteratorRecord#next() pushing ${this.val}`);
          return {
            value: this.val,
            done: this.val++ === 10 ? true : false,
          };
        },
        return() {
          results.push(`IteratorRecord#return() called with this.val=${this.val}`);
        },
      };
    },
  };

  const ac = new AbortController();
  Observable.from(iterable).subscribe(v => {
    results.push(`Observing ${v}`);
    if (v === 3) {
      ac.abort();
    }
  }, {signal: ac.signal});

  assert_array_equals(results, [
    "IteratorRecord#next() pushing 0",
    "Observing 0",
    "IteratorRecord#next() pushing 1",
    "Observing 1",
    "IteratorRecord#next() pushing 2",
    "Observing 2",
    "IteratorRecord#next() pushing 3",
    "Observing 3",
    "IteratorRecord#return() called with this.val=4",
  ]);
}, "from(): Aborting sync iterable midway through iteration both stops iteration " +
   "and invokes `IteratorRecord#return()");
// Like the above test, but for async iterables.
promise_test(async t => {
  const results = [];
  const iterable = {
    [Symbol.asyncIterator]() {
      return {
        val: 0,
        next() {
          results.push(`IteratorRecord#next() pushing ${this.val}`);
          return {
            value: this.val,
            done: this.val++ === 10 ? true : false,
          };
        },
        return(reason) {
          results.push(`IteratorRecord#return() called with reason=${reason}`);
          return {done: true};
        },
      };
    },
  };

  const ac = new AbortController();
  await new Promise(resolve => {
    Observable.from(iterable).subscribe(v => {
      results.push(`Observing ${v}`);
      if (v === 3) {
        ac.abort(`Aborting because v=${v}`);
        resolve();
      }
    }, {signal: ac.signal});
  });

  assert_array_equals(results, [
    "IteratorRecord#next() pushing 0",
    "Observing 0",
    "IteratorRecord#next() pushing 1",
    "Observing 1",
    "IteratorRecord#next() pushing 2",
    "Observing 2",
    "IteratorRecord#next() pushing 3",
    "Observing 3",
    "IteratorRecord#return() called with reason=Aborting because v=3",
  ]);
}, "from(): Aborting async iterable midway through iteration both stops iteration " +
   "and invokes `IteratorRecord#return()");

test(() => {
  const iterable = {
    [Symbol.iterator]() {
      return {
        val: 0,
        next() {
          return {value: this.val, done: this.val++ === 10 ? true : false};
        },
        // Not returning an Object results in a TypeError being thrown.
        return(reason) {},
      };
    },
  };

  let thrownError = null;
  const ac = new AbortController();
  Observable.from(iterable).subscribe(v => {
    if (v === 3) {
      try {
        ac.abort(`Aborting because v=${v}`);
      } catch (e) {
        thrownError = e;
      }
    }
  }, {signal: ac.signal});

  assert_not_equals(thrownError, null, "abort() threw an Error");
  assert_true(thrownError instanceof TypeError);
  assert_true(thrownError.message.includes('return()'));
  assert_true(thrownError.message.includes('Object'));
}, "from(): Sync iterable: `Iterator#return()` must return an Object, or an " +
   "error is thrown");
// This test is just like the above but for async iterables. It asserts that a
// Promise is rejected when `return()` does not return an Object.
promise_test(async t => {
  const iterable = {
    [Symbol.asyncIterator]() {
      return {
        val: 0,
        next() {
          return {value: this.val, done: this.val++ === 10 ? true : false};
        },
        // Not returning an Object results in a rejected Promise.
        return(reason) {},
      };
    },
  };

  const unhandled_rejection_promise = new Promise((resolve, reject) => {
    const unhandled_rejection_handler = e => resolve(e.reason);
    self.addEventListener("unhandledrejection", unhandled_rejection_handler);
    t.add_cleanup(() =>
        self.removeEventListener("unhandledrejection", unhandled_rejection_handler));

    t.step_timeout(() => reject('Timeout'), 3000);
  });

  const ac = new AbortController();
  await new Promise(resolve => {
    Observable.from(iterable).subscribe(v => {
      if (v === 3) {
        ac.abort(`Aborting because v=${v}`);
        resolve();
      }
    }, {signal: ac.signal});
  });

  const reason = await unhandled_rejection_promise;
  assert_true(reason instanceof TypeError);
  assert_true(reason.message.includes('return()'));
  assert_true(reason.message.includes('Object'));
}, "from(): Async iterable: `Iterator#return()` must return an Object, or a " +
   "Promise rejects asynchronously");

// This test exercises the logic of `GetIterator()` async->sync fallback
// logic. Specifically, we have an object that is an async iterable — that is,
// it has a callback [Symbol.asyncIterator] implementation. Observable.from()
// detects this, and commits to converting the object from the async iterable
// protocol. Then, after conversion but before subscription, the object is
// modified such that it no longer implements the async iterable protocol.
//
// But since it still implements the *iterable* protocol, ECMAScript's
// `GetIterator()` abstract algorithm [1] is fully exercised, which is spec'd to
// fall-back to the synchronous iterable protocol if it exists, and create a
// fully async iterable out of the synchronous iterable.
//
// [1]: https://tc39.es/ecma262/#sec-getiterator
promise_test(async () => {
  const results = [];
  const async_iterable = {
    asyncIteratorGotten: false,
    get [Symbol.asyncIterator]() {
      results.push("[Symbol.asyncIterator] GETTER");
      if (this.asyncIteratorGotten) {
        return null; // Both null and undefined work here.
      }

      this.asyncIteratorGotten = true;
      // The only requirement for `this` to be converted as an async
      // iterable -> Observable is that the return value be callable (i.e., a function).
      return function() {};
    },

    [Symbol.iterator]() {
      results.push('[Symbol.iterator]() invoked as fallback');
      return {
        val: 0,
        next() {
          return {
            value: this.val,
            done: this.val++ === 4 ? true : false,
          };
        },
      };
    },
  };

  const source = Observable.from(async_iterable);
  assert_array_equals(results, [
    "[Symbol.asyncIterator] GETTER",
  ]);

  await new Promise((resolve, reject) => {
    source.subscribe({
      next: v => {
        results.push(`Observing ${v}`);
        queueMicrotask(() => results.push(`next() microtask interleaving (v=${v})`));
      },
      error: e => reject(e),
      complete: () => {
        results.push('complete()');
        resolve();
      },
    });
  });

  assert_array_equals(results, [
    // Old:
    "[Symbol.asyncIterator] GETTER",
    // New:
    "[Symbol.asyncIterator] GETTER",
    "[Symbol.iterator]() invoked as fallback",
    "Observing 0",
    "next() microtask interleaving (v=0)",
    "Observing 1",
    "next() microtask interleaving (v=1)",
    "Observing 2",
    "next() microtask interleaving (v=2)",
    "Observing 3",
    "next() microtask interleaving (v=3)",
    "complete()",
  ]);
}, "from(): Asynchronous iterable conversion, with synchronous iterable fallback");

test(() => {
  const results = [];
  let generatorFinalized = false;

  const generator = function*() {
    try {
      for (let n = 0; n < 10; n++) {
        yield n;
      }
    } finally {
      generatorFinalized = true;
    }
  };

  const observable = Observable.from(generator());
  const abortController = new AbortController();

  observable.subscribe(n => {
    results.push(n);
    if (n === 3) {
      abortController.abort();
    }
  }, {signal: abortController.signal});

  assert_array_equals(results, [0, 1, 2, 3]);
  assert_true(generatorFinalized);
}, "from(): Generator finally block runs when subscription is aborted");

test(() => {
  const results = [];
  let generatorFinalized = false;

  const generator = function*() {
    try {
      for (let n = 0; n < 10; n++) {
        yield n;
      }
    } catch {
      assert_unreached("generator should not be aborted");
    } finally {
      generatorFinalized = true;
    }
  };

  const observable = Observable.from(generator());

  observable.subscribe((n) => {
    results.push(n);
  });

  assert_array_equals(results, [0, 1, 2, 3, 4, 5, 6, 7, 8, 9]);
  assert_true(generatorFinalized);
}, "from(): Generator finally block run when Observable completes");

test(() => {
  const results = [];
  let generatorFinalized = false;

  const generator = function*() {
    try {
      for (let n = 0; n < 10; n++) {
        yield n;
      }
      throw new Error('from the generator');
    } finally {
      generatorFinalized = true;
    }
  };

  const observable = Observable.from(generator());

  observable.subscribe({
    next: n => results.push(n),
    error: e => results.push(e.message)
  });

  assert_array_equals(results, [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, "from the generator"]);
  assert_true(generatorFinalized);
}, "from(): Generator finally block run when Observable errors");

promise_test(async t => {
  const results = [];
  let generatorFinalized = false;

  async function* asyncGenerator() {
    try {
      for (let n = 0; n < 10; n++) {
        yield n;
      }
    } finally {
      generatorFinalized = true;
    }
  }

  const observable = Observable.from(asyncGenerator());
  const abortController = new AbortController();

  await new Promise((resolve) => {
    observable.subscribe((n) => {
      results.push(n);
      if (n === 3) {
        abortController.abort();
        resolve();
      }
    }, {signal: abortController.signal});
  });

  assert_array_equals(results, [0, 1, 2, 3]);
  assert_true(generatorFinalized);
}, "from(): Async generator finally block run when subscription is aborted");

promise_test(async t => {
  const results = [];
  let generatorFinalized = false;

  async function* asyncGenerator() {
    try {
      for (let n = 0; n < 10; n++) {
        yield n;
      }
    } finally {
      generatorFinalized = true;
    }
  }

  const observable = Observable.from(asyncGenerator());

  await new Promise(resolve => {
    observable.subscribe({
      next: n => results.push(n),
      complete: () => resolve(),
    });
  });

  assert_array_equals(results, [0, 1, 2, 3, 4, 5, 6, 7, 8, 9]);
  assert_true(generatorFinalized);
}, "from(): Async generator finally block runs when Observable completes");

promise_test(async t => {
  const results = [];
  let generatorFinalized = false;

  async function* asyncGenerator() {
    try {
      for (let n = 0; n < 10; n++) {
        if (n === 4) {
          throw new Error('from the async generator');
        }
        yield n;
      }
    } finally {
      generatorFinalized = true;
    }
  }

  const observable = Observable.from(asyncGenerator());

  await new Promise((resolve) => {
    observable.subscribe({
      next: (n) => results.push(n),
      error: (e) => {
        results.push(e.message);
        resolve();
      }
    });
  });

  assert_array_equals(results, [0, 1, 2, 3, "from the async generator"]);
  assert_true(generatorFinalized);
}, "from(): Async generator finally block run when Observable errors");

// Test what happens when `return()` throws an error upon abort.
test(() => {
  const results = [];
  const iterable = {
    [Symbol.iterator]() {
      return {
        val: 0,
        next() {
          results.push('next() called');
          return {value: this.val, done: this.val++ === 10 ? true : false};
        },
        return() {
          results.push('return() about to throw an error');
          throw new Error('return() error');
        },
      };
    }
  };

  const ac = new AbortController();
  const source = Observable.from(iterable);
  source.subscribe(v => {
    if (v === 3) {
      try {
        ac.abort();
      } catch (e) {
        results.push(`AbortController#abort() threw an error: ${e.message}`);
      }
    }
  }, {signal: ac.signal});

  assert_array_equals(results, [
    'next() called',
    'next() called',
    'next() called',
    'next() called',
    'return() about to throw an error',
    'AbortController#abort() threw an error: return() error',
  ]);
}, "from(): Sync iterable: error thrown from IteratorRecord#return() can be " +
   "synchronously caught");
promise_test(async t => {
  const results = [];
  const iterable = {
    [Symbol.asyncIterator]() {
      return {
        val: 0,
        next() {
          results.push('next() called');
          return {value: this.val, done: this.val++ === 10 ? true : false};
        },
        return() {
          results.push('return() about to throw an error');
          // For async iterables, errors thrown in `return()` end up in a
          // returned rejected Promise, so no error appears on the stack
          // immediately. See [1].
          //
          // [1]: https://whatpr.org/webidl/1397.html#async-iterator-close.
          throw new Error('return() error');
        },
      };
    }
  };

  const unhandled_rejection_promise = new Promise((resolve, reject) => {
    const unhandled_rejection_handler = e => resolve(e.reason);
    self.addEventListener("unhandledrejection", unhandled_rejection_handler);
    t.add_cleanup(() =>
        self.removeEventListener("unhandledrejection", unhandled_rejection_handler));

    t.step_timeout(() => reject('Timeout'), 1500);
  });

  const ac = new AbortController();
  const source = Observable.from(iterable);
  await new Promise((resolve, reject) => {
    source.subscribe(v => {
      if (v === 3) {
        try {
          ac.abort();
          results.push('No error thrown synchronously');
          resolve('No error thrown synchronously');
        } catch (e) {
          results.push(`AbortController#abort() threw an error: ${e.message}`);
          reject(e);
        }
      }
    }, {signal: ac.signal});
  });

  assert_array_equals(results, [
    'next() called',
    'next() called',
    'next() called',
    'next() called',
    'return() about to throw an error',
    'No error thrown synchronously',
  ]);

  const reason = await unhandled_rejection_promise;
  assert_true(reason instanceof Error);
  assert_equals(reason.message, "return() error",
      "Custom error text passed through rejected Promise");
}, "from(): Async iterable: error thrown from IteratorRecord#return() is " +
   "wrapped in rejected Promise");

test(() => {
  const results = [];
  const iterable = {
    impl() {
      return {
        next() {
          results.push('next() running');
          return {done: true};
        }
      };
    }
  };

  iterable[Symbol.iterator] = iterable.impl;
  {
    const source = Observable.from(iterable);
    source.subscribe({}, {signal: AbortSignal.abort()});
    assert_array_equals(results, []);
  }
  iterable[Symbol.iterator] = undefined;
  iterable[Symbol.asyncIterator] = iterable.impl;
  {
    const source = Observable.from(iterable);
    source.subscribe({}, {signal: AbortSignal.abort()});
    assert_array_equals(results, []);
  }
}, "from(): Subscribing to an iterable Observable with an aborted signal " +
   "does not call next()");

test(() => {
  const results = [];
  const ac = new AbortController();

  const iterable = {
    [Symbol.iterator]() {
      ac.abort();
      return {
        val: 0,
        next() {
          results.push('next() called');
          return {done: true};
        },
        return() {
          results.push('return() called');
        }
      };
    }
 };

  const source = Observable.from(iterable);
  source.subscribe({
    next: v => results.push(v),
    complete: () => results.push('complete'),
  }, {signal: ac.signal});

  assert_array_equals(results, []);
}, "from(): When iterable conversion aborts the subscription, next() is " +
   "never called");
test(() => {
  const results = [];
  const ac = new AbortController();

  const iterable = {
    [Symbol.asyncIterator]() {
      ac.abort();
      return {
        val: 0,
        next() {
          results.push('next() called');
          return {done: true};
        },
        return() {
          results.push('return() called');
        }
      };
    }
  };

  const source = Observable.from(iterable);
  source.subscribe({
    next: v => results.push(v),
    complete: () => results.push('complete'),
  }, {signal: ac.signal});

  assert_array_equals(results, []);
}, "from(): When async iterable conversion aborts the subscription, next() " +
   "is never called");

// This test asserts some very subtle behavior with regard to async iterables
// and a mid-subscription signal abort. Specifically it detects that a signal
// abort ensures that the `next()` method is not called again on the iterator
// again, BUT detects that pending Promise from the *previous* `next()` call
// still has its IteratorResult object examined. I.e., the implementation
// inspecting the `done` attribute on the resolved IteratorResult is observable
// event after abort() takes place.
promise_test(async () => {
  const results = [];
  let resolveNext = null;

  const iterable = {
    [Symbol.asyncIterator]() {
      return {
        next() {
          results.push('next() called');
          return new Promise(resolve => {
            resolveNext = resolve;
          });
        },
        return() {
          results.push('return() called');
        }
      };
    }
  };

  const ac = new AbortController();
  const source = Observable.from(iterable);
  source.subscribe({
    next: v => results.push(v),
    complete: () => results.push('complete'),
  }, {signal: ac.signal});

  assert_array_equals(results, [
    "next() called",
  ]);

  // First abort, ensuring `return()` is called.
  ac.abort();

  assert_array_equals(results, [
    "next() called",
    "return() called",
  ]);

  // Then resolve the pending `next()` Promise to an object whose `done` getter
  // reports to the test whether it was accessed. We have to wait one microtask
  // for the internal Observable implementation to finish "reacting" to said
  // `next()` promise resolution, for it to grab the `done` attribute.
  await new Promise(resolveOuter => {
    resolveNext({
      get done() {
        results.push('IteratorResult.done GETTER');
        resolveOuter();
        return true;
      }
    });
  });

  assert_array_equals(results, [
    "next() called",
    "return() called",
    "IteratorResult.done GETTER",
    // Note that "next() called" does not make another appearance.
  ]);
}, "from(): Aborting an async iterable subscription stops subsequent next() " +
   "calls, but old next() Promise reactions are web-observable");

test(() => {
  const results = [];
  const iterable = {
    [Symbol.iterator]() {
      return {
        val: 0,
        next() {
          return {value: this.val, done: this.val++ === 4 ? true : false};
        },
        return() {
          results.push('return() called');
        },
      };
    }
  };

  const source = Observable.from(iterable);
  const ac = new AbortController();
  source.subscribe({
    next: v => results.push(v),
    complete: () => results.push('complete'),
  }, {signal: ac.signal});

  ac.abort(); // Must do nothing!
  assert_array_equals(results, [0, 1, 2, 3, 'complete']);
}, "from(): Abort after complete does NOT call IteratorRecord#return()");
