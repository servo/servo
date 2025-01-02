promise_test(async t => {
  const { checkMicrotask } = await import("./sibling-imports-not-blocked__microtask__parent.js");

  assert_equals(checkMicrotask, "PASS");
}, "Async modules only scheduling microtasks don't block execution of sibling modules");

promise_test(async t => {
  const { checkTask } = await import("./sibling-imports-not-blocked__task__parent.js");

  assert_equals(checkTask, "PASS");
}, "Async modules scheduling tasks don't block execution of sibling modules");
