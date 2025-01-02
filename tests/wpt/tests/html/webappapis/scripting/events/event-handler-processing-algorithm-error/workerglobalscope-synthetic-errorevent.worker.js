"use strict";
importScripts("/resources/testharness.js");

setup({ allow_uncaught_exception: true });

promise_test(t => {
  self.onerror = t.step_func((...args) => {
    assert_greater_than(args.length, 1);
    return true;
  });

  const eventWatcher = new EventWatcher(t, self, "error");
  const promise = eventWatcher.wait_for("error").then(e => {
    assert_equals(e.defaultPrevented, true);
  });

  self.dispatchEvent(new ErrorEvent("error", { cancelable: true }));

  return promise;
}, "error event is weird (return true cancels; many args) on WorkerGlobalScope, with a synthetic ErrorEvent");

promise_test(t => {
  const theError = { the: "error object" };

  self.onerror = t.step_func(function (message, filename, lineno, colno, error) {
    assert_equals(arguments.length, 5, "There must be exactly 5 arguments");
    assert_equals(message, "message");
    assert_equals(filename, "filename");
    assert_equals(lineno, 1);
    assert_equals(colno, 2);
    assert_equals(error, theError);
    return true;
  });

  const eventWatcher = new EventWatcher(t, self, "error");
  const promise = eventWatcher.wait_for("error");

  self.dispatchEvent(new ErrorEvent("error", {
    message: "message",
    filename: "filename",
    lineno: 1,
    colno: 2,
    error: theError
  }));

  return promise;
}, "error event has the right 5 args on WorkerGlobalScope, with a synthetic ErrorEvent");

done();
