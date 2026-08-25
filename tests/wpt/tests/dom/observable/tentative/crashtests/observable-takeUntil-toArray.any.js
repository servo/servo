// Because we test that the global error handler is called at various times.
promise_test(async () => {
  return new Observable(() => {})
      .takeUntil(new Observable(() => {})
          .inspect({subscribe: () => { throw "KABOOM"; }}))
      .toArray();
}, "takeUntil correctly passes error on inner observable to outer observables");

