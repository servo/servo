// Testing that the resolution is correct using `resolve`, as you can't import
// the same module twice.
window.inscope_test_result = import.meta.resolve("a");
