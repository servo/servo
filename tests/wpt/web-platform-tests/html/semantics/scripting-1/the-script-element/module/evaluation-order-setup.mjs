globalThis.setup({allow_uncaught_exception: true});

globalThis.log = [];

globalThis.addEventListener("error",
    event => globalThis.log.push("global-error", event.error.message));
globalThis.addEventListener("onunhandledrejection",
    event => globalThis.log.push('unhandled-promise-rejection'));
globalThis.addEventListener("load",
    event => globalThis.log.push("global-load"));

globalThis.unreachable = function() {
    globalThis.log.push("unreachable");
}

globalThis.test_load = async_test("Test evaluation order of modules");
globalThis.testDone = globalThis.test_load.step_func_done(() => {
  assert_array_equals(globalThis.log, globalThis.expectedLog);
});
