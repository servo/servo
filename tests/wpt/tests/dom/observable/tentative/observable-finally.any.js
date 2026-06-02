// Because we test that the global error handler is called at various times.
setup({allow_uncaught_exception: true});

test(() => {
  const source = new Observable((subscriber) => {
    subscriber.next(1);
    subscriber.next(2);
    subscriber.next(3);
    subscriber.complete();
  });

  const results = [];

  source
    .finally(() => {
      results.push("finally called");
    })
    .subscribe({
      next: (value) => results.push(value),
      error: (e) => results.push(e.message),
      complete: () => results.push("complete"),
    });

  assert_array_equals(results, [1, 2, 3, "finally called", "complete"],
      "finally is called with teardown timing, before complete() is forwarded");
}, "finally(): Mirrors all values and completions from source");

test(() => {
  const source = new Observable((subscriber) => {
    subscriber.next(1);
    subscriber.next(2);
    subscriber.next(3);
    subscriber.error(new Error("error from source"));
  });

  const results = [];

  source
    .finally(() => {
      results.push("finally called");
    })
    .subscribe({
      next: (value) => results.push(value),
      error: (e) => results.push(e.message),
      complete: () => results.push("complete"),
    });

  assert_array_equals(results, [1, 2, 3, "finally called", "error from source"],
      "finally is called with teardown timing, before complete() is forwarded");
}, "finally(): Mirrors all values and errors from the source");

test(() => {
  const results = [];

  const source = new Observable((subscriber) => {
    results.push("source subscribe");
    subscriber.addTeardown(() => results.push("source teardown"));
    results.push("source send complete");
    subscriber.complete();
  });

  const result = source.finally(() => {
    results.push("finally handler");
  });

  result.subscribe({
    complete: () => results.push("result complete"),
  });

  assert_array_equals(results, [
    "source subscribe",
    "source send complete",
    "source teardown",
    "finally handler",
    "result complete",
  ]);
}, "finally(): Callback handler fires BEFORE the source observable completes");

test(() => {
  const results = [];

  const source = new Observable((subscriber) => {
    results.push("source subscribe");
    subscriber.addTeardown(() => results.push("source teardown"));
    results.push("source send error");
    subscriber.error(new Error("error from source"));
  });

  const result = source.finally(() => {
    results.push("finally handler");
  });

  result.subscribe({
    error: (e) => results.push(e.message),
  });

  assert_array_equals(results, [
    "source subscribe",
    "source send error",
    "source teardown",
    "finally handler",
    "error from source",
  ]);
}, "finally(): Callback handler fires BEFORE the source observable errors");

test(() => {
  const results = [];

  const source = new Observable((subscriber) => {
    subscriber.complete();
  });

  const result = source
    .finally(() => {
      results.push("finally handler 1");
    })
    .finally(() => {
      results.push("finally handler 2");
    });

  result.subscribe({ complete: () => results.push("result complete") });

  assert_array_equals(results,
    ["finally handler 1", "finally handler 2", "result complete"]);
}, "finally(): Handlers run in composition order");

test(() => {
  const source = new Observable(subscriber => {
    subscriber.error("producer error");
  });

  const results = [];

  self.addEventListener('error', e => results.push(e.error.message), {once: true});

  source
    .finally(() => {
      throw new Error("error from finally");
    })
    .subscribe({
      next: () => results.push("next"),
      error: (e) => results.push(e),
      complete: () => results.push("complete"),
    });

  assert_array_equals(results, ["error from finally", "producer error"]);
}, "finally(): Errors thrown in the finally handler " +
   "(during Subscriber#error()) are reported to the global immediately");

test(() => {
  const source = new Observable((subscriber) => {
    subscriber.complete();
  });

  const results = [];

  self.addEventListener('error', e => results.push(e.error.message), {once: true});

  source
    .finally(() => {
      throw new Error("error from finally");
    })
    .subscribe({
      next: () => results.push("next"),
      error: (e) => results.push("unreached"),
      complete: () => results.push("complete"),
    });

  assert_array_equals(results, ["error from finally", "complete"]);
}, "finally(): Errors thrown in the finally handler " +
   "(during Subscriber#complete()) are reported to the global immediately");

test(() => {
  const results = [];

  const source = new Observable((subscriber) => {
    subscriber.addTeardown(() => results.push("source teardown"));
  });

  const controller = new AbortController();

  source
    .finally(() => results.push("downstream finally handler"))
    .subscribe({}, { signal: controller.signal });

  controller.abort();

  assert_array_equals(results, ["source teardown", "downstream finally handler"]);
}, "finally(): Callback is run if consumer aborts the subscription");

test(() => {
  const results = [];
  const result = new Observable((subscriber) => {
    subscriber.next(1);
    subscriber.next(2);
    subscriber.complete();
  }).flatMap((value) => {
    results.push(`flatMap ${value}`);
    return new Observable((subscriber) => {
      subscriber.next(value);
      subscriber.next(value);
      subscriber.next(value);
      subscriber.complete();
    }).finally(() => {
      results.push(`finally ${value}`);
    });
  });

  result.subscribe({
    next: (value) => results.push(`result ${value}`),
    complete: () => results.push("result complete"),
  });

  assert_array_equals(results, [
    "flatMap 1",
    "result 1",
    "result 1",
    "result 1",
    "finally 1",
    "flatMap 2",
    "result 2",
    "result 2",
    "result 2",
    "finally 2",
    "result complete",
  ]);
}, "finally(): Callback is run before next inner subscription in flatMap()");

test(() => {
  const results = [];
  const result = new Observable((subscriber) => {
    subscriber.next(1);
    subscriber.next(2);
    subscriber.complete();
  }).switchMap((value) => {
    results.push(`switchMap ${value}`);
    return new Observable((subscriber) => {
      subscriber.next(value);
      subscriber.next(value);
      subscriber.next(value);
      subscriber.complete();
    }).finally(() => {
      results.push(`finally ${value}`);
    });
  });

  result.subscribe({
    next: (value) => results.push(`result ${value}`),
    complete: () => results.push("result complete"),
  });

  assert_array_equals(results, [
    "switchMap 1",
    "result 1",
    "result 1",
    "result 1",
    "finally 1",
    "switchMap 2",
    "result 2",
    "result 2",
    "result 2",
    "finally 2",
    "result complete",
  ]);
}, "finally(): Callback is run before next inner subscription in switchMap()");
