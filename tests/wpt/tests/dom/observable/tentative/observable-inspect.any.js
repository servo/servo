// Because we test that the global error handler is called at various times.
setup({ allow_uncaught_exception: true });

test(() => {
  const results = [];
  let sourceSubscriptionCall = 0;
  const source = new Observable(subscriber => {
    sourceSubscriptionCall++;
    results.push(`source subscribe ${sourceSubscriptionCall}`);
    subscriber.next(1);
    subscriber.next(2);
    subscriber.next(3);
    subscriber.complete();
  });

  let inspectSubscribeCall = 0;
  const result = source.inspect({
    subscribe: () => {
      inspectSubscribeCall++;
      results.push(`inspect() subscribe ${inspectSubscribeCall}`);
    },
    next: (value) => results.push(`inspect() next ${value}`),
    error: (e) => results.push(`inspect() error ${e.message}`),
    complete: () => results.push(`inspect() complete`),
  });

  result.subscribe({
    next: (value) => results.push(`result next ${value}`),
    error: (e) => results.push(`result error ${e.message}`),
    complete: () => results.push(`result complete`),
  });

  result.subscribe({
    next: (value) => results.push(`result next ${value}`),
    error: (e) => results.push(`result error ${e.message}`),
    complete: () => results.push(`result complete`),
  });

  assert_array_equals(results,
    [
      "inspect() subscribe 1",
      "source subscribe 1",
      "inspect() next 1",
      "result next 1",
      "inspect() next 2",
      "result next 2",
      "inspect() next 3",
      "result next 3",
      "inspect() complete",
      "result complete",
      "inspect() subscribe 2",
      "source subscribe 2",
      "inspect() next 1",
      "result next 1",
      "inspect() next 2",
      "result next 2",
      "inspect() next 3",
      "result next 3",
      "inspect() complete",
      "result complete",
    ]);
}, "inspect(): Provides a pre-subscription subscribe callback");

test(() => {
  const source = new Observable(subscriber => {
    subscriber.next(1);
    subscriber.next(2);
    subscriber.next(3);
    subscriber.complete();
  });

  const results = [];

  const result = source.inspect({
    next: value => results.push(value),
    error: e => results.push(e),
    complete: () => results.push("complete"),
  });

  result.subscribe();
  result.subscribe();

  assert_array_equals(results, [1, 2, 3, "complete", 1, 2, 3, "complete"]);
}, "inspect(): Provides a way to tap into the values and completions of the " +
   "source observable using an observer");

test(() => {
  const error = new Error("error from source");
  const source = new Observable(subscriber => subscriber.error(error));

  const results = [];

  const result = source.inspect({
    next: value => results.push(value),
    error: e => results.push(e),
    complete: () => results.push("complete"),
  });

  let errorReported = null;
  self.addEventListener('error', e => errorReported = e.error, {once: true});
  result.subscribe();

  assert_array_equals(results, [error]);
  assert_equals(errorReported, error,
      "errorReported to global matches error from source Observable");
}, "inspect(): Error handler does not stop error from being reported to the " +
   "global, when subscriber does not pass error handler");

test(() => {
  const error = new Error("error from source");
  const source = new Observable(subscriber => {
    subscriber.next(1);
    subscriber.next(2);
    subscriber.next(3);
    subscriber.error(error);
  });

  const results = [];

  const result = source.inspect({
    next: value => results.push(value),
    error: e => results.push(e),
    complete: () => results.push("complete"),
  });

  const observer = {
    error: e => results.push(e),
  };
  result.subscribe(observer);
  result.subscribe(observer);

  assert_array_equals(results, [1, 2, 3, error, error, 1, 2, 3, error, error]);
}, "inspect(): Provides a way to tap into the values and errors of the " +
   "source observable using an observer. Errors are passed through");

test(() => {
  const source = new Observable(subscriber => {
    subscriber.next(1);
    subscriber.next(2);
    subscriber.next(3);
    subscriber.complete();
  });

  const results = [];

  const result = source.inspect(value => results.push(value));

  result.subscribe();
  result.subscribe();

  assert_array_equals(results, [1, 2, 3, 1, 2, 3]);
}, "inspect(): ObserverCallback passed in");

test(() => {
  const source = new Observable(subscriber => {
    subscriber.next(1);
    subscriber.next(2);
    subscriber.next(3);
  });

  const error = new Error("error from inspect() next handler");
  const result = source.inspect({
    next: (value) => {
      if (value === 2) {
        throw error;
      }
    },
  });

  const results1 = [];
  result.subscribe({
    next: (value) => results1.push(value),
    error: (e) => results1.push(e),
    complete: () => results1.push("complete"),
  });

  const results2 = [];
  result.subscribe({
    next: (value) => results2.push(value),
    error: (e) => results2.push(e),
    complete: () => results2.push("complete"),
  });

  assert_array_equals(results1, [1, error]);
  assert_array_equals(results2, [1, error]);
}, "inspect(): Throwing an error in the observer next handler is caught and " +
   "sent to the error callback of the result observable");

test(() => {
  const sourceError = new Error("error from source");
  const inspectError = new Error("error from inspect() error handler");

  const source = new Observable(subscriber => {
    subscriber.error(sourceError);
  });

  const result = source.inspect({
    error: () => {
      throw inspectError;
    },
  });

  const results = [];
  result.subscribe({
    next: () => results.push("next"),
    error: (e) => results.push(e),
    complete: () => results.push("complete"),
  });

  assert_array_equals(results, [inspectError]);
}, "inspect(): Throwing an error in the observer error handler in " +
   "inspect() is caught and sent to the error callback of the result " +
   "observable");

test(() => {
  const source = new Observable(subscriber => {
    subscriber.next(1);
    subscriber.next(2);
    subscriber.next(3);
    subscriber.complete();
  });

  const error = new Error("error from inspect() complete handler");
  const result = source.inspect({
    complete: () => {
      throw error;
    },
  });

  const results = [];
  result.subscribe({
    next: (value) => results.push(value),
    error: (e) => results.push(e),
    complete: () => results.push("complete"),
  });

  assert_array_equals(results, [1, 2, 3, error]);
}, "inspect(): Throwing an error in the observer complete handler is caught " +
   "and sent to the error callback of the result observable");

test(() => {
  const source = new Observable(subscriber => {
    subscriber.next(1);
    subscriber.next(2);
    subscriber.next(3);
  });

  const error = new Error("error from inspect() next handler");
  const result = source.inspect({
    next: (value) => {
      if (value === 2) {
        throw error;
      }
    },
  });

  const results = [];
  result.subscribe({
    next: (value) => results.push(value),
    error: (e) => results.push(e),
    complete: () => results.push("complete"),
  });

  assert_array_equals(results, [1, error]);
}, "inspect(): Throwing an error in the next handler function in do should " +
   "be caught and sent to the error callback of the result observable");

test(() => {
  const source = new Observable(subscriber => {});

  const result = source.inspect({
    subscribe: () => {
      throw new Error("error from do subscribe handler");
    },
  });

  const results = [];
  result.subscribe({
    next: () => results.push("next"),
    error: (e) => results.push(e.message),
    complete: () => results.push("complete"),
  });

  assert_array_equals(results, ["error from do subscribe handler"]);
}, "inspect(): Errors thrown in subscribe() Inspector handler subscribe " +
   "handler are caught and sent to error callback");

test(() => {
  const results = [];
  let sourceTeardownCall = 0;
  const source = new Observable(subscriber => {
    subscriber.addTeardown(() => {
      sourceTeardownCall++;
      results.push(`source teardown ${sourceTeardownCall}`);
    });
    subscriber.next(1);
    subscriber.next(2);
    subscriber.next(3);
    subscriber.complete();
  });

  let doUnsubscribeCall = 0;
  const result = source.inspect({
    abort: (reason) => {
      doUnsubscribeCall++;
      results.push(`inspect() abort ${doUnsubscribeCall} ${reason}`);
    },
    next: (value) => results.push(`inspect() next ${value}`),
    error: (e) => results.push(`inspect() error ${e.message}`),
    complete: () => results.push(`inspect() complete`),
  });

  const controller = new AbortController();
  result.subscribe({
    next: (value) => {
      results.push(`result next ${value}`);
      if (value === 2) {
        controller.abort("abort reason");
      }
    },
    error: (e) => results.push(`result error ${e.message}`),
    complete: () => results.push(`result complete`),
  }, { signal: controller.signal });

  assert_array_equals(results, [
    "inspect() next 1",
    "result next 1",
    "inspect() next 2",
    "result next 2",
    "inspect() abort 1 abort reason",
    "source teardown 1",
  ]);
}, "inspect(): Provides a way to tap into the moment a source observable is unsubscribed from");

test(() => {
  const results = [];
  let sourceTeardownCall = 0;
  const source = new Observable(subscriber => {
    subscriber.addTeardown(() => {
      sourceTeardownCall++;
      results.push(`source teardown ${sourceTeardownCall}`);
    });
    subscriber.next(1);
    subscriber.next(2);
    subscriber.next(3);
    subscriber.complete();
  });

  let inspectUnsubscribeCall = 0;
  const result = source.inspect({
    next: (value) => results.push(`inspect() next ${value}`),
    complete: () => results.push(`inspect() complete`),
    abort: (reason) => {
      inspectUnsubscribeCall++;
      results.push(`inspect() abort ${inspectUnsubscribeCall} ${reason}`);
    },
  });

  result.subscribe({
    next: (value) => results.push(`result next ${value}`),
    complete: () => results.push(`result complete`),
  });

  assert_array_equals(results, [
    "inspect() next 1",
    "result next 1",
    "inspect() next 2",
    "result next 2",
    "inspect() next 3",
    "result next 3",
    "source teardown 1",
    "inspect() complete",
    "result complete",
  ]);
}, "inspect(): Inspector abort() handler is not called if the source " +
   "completes before the result is unsubscribed from");

test(() => {
  const source = new Observable(subscriber => {
    subscriber.next(1);
  });

  const results = [];

  const result = source.inspect({
    abort: () => {
      results.push('abort() handler run');
      throw new Error("error from inspect() subscribe handler");
    },
  });

  const controller = new AbortController();

  self.when('error').take(1).subscribe(e =>
      results.push(e.message + ', from report exception'));

  result.subscribe({
    next: (value) => {
      results.push(value);
      controller.abort();
    },
    // This should not be invoked at all!!
    error: (e) => results.push(e.message + ', from Observer#error()'),
    complete: () => results.push("complete"),
  }, {signal: controller.signal});

  assert_array_equals(results, [1, "abort() handler run",
      "Uncaught Error: error from inspect() subscribe handler, from report " +
      "exception"]);
}, "inspect(): Errors thrown from inspect()'s abort() handler are caught " +
   "and reported to the global, because the subscription is already closed " +
   "by the time the handler runs");
