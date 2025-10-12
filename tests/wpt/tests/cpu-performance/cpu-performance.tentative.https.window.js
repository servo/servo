test(function () {
  assert_equals(typeof navigator.cpuPerformance, "number",
    "navigator.cpuPerformance returns a number");
  assert_true(Number.isInteger(navigator.cpuPerformance),
    "navigator.cpuPerformance returns an integer value");
  assert_true(navigator.cpuPerformance >= 0,
    "navigator.cpuPerformance returns a non-negative value");
}, "navigator.cpuPerformance is valid, if the feature is enabled");
