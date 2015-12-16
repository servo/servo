importScripts("/resources/testharness.js");

test(function() {
  assert_true((performance !== undefined), "WorkerGlobalScope.performance exists");
  assert_equals((typeof performance.now), "function");
}, "WorkerGlobalScope.performance.now() is a function");

test(function() {
  assert_true(performance.now() > 0);
}, "WorkerGlobalScope.performance.now() returns a positive number");

test(function() {
    var now1 = performance.now();
    var now2 = performance.now();
    assert_true((now2-now1) >= 0);
  }, "WorkerGlobalScope.performance.now() difference is not negative");

done();
