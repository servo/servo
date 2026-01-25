function isPowerOfTwo(n) {
  const log2 = Math.log2(n);
  return log2 == Math.ceil(log2);
}

// Note the spec suggests applying implementation-defined min and max limits
// to reduce fingerprinting risk for less common memory configurations.
// However, it does not recommend limits so these are not tested in WPT.
// These should be tested by implementors.
test(function() {
    assert_equals(typeof navigator.deviceMemory, "number",
        "navigator.deviceMemory returns a number");
    assert_true(navigator.deviceMemory >= 0,
        "navigator.deviceMemory returns a positive value");
    assert_true(isPowerOfTwo(navigator.deviceMemory),
        "navigator.deviceMemory returns a power of 2");
}, "navigator.deviceMemory is a positive number, a power of 2");
