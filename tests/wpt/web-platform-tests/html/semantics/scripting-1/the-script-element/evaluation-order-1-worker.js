globalThis.expectedLog = [
  "step-1-1", "step-1-2",
  "global-error", "error",
  "microtask",
];

globalThis.test_load.step_timeout(() => globalThis.testDone(), 0);

done();
