importScripts("/resources/testharness.js");

// Regression test for https://github.com/web-platform-tests/wpt/issues/27299,
// where we broke the ability for a setup function in a worker to contain an
// assertion (even a passing one).
setup(function() {
    assert_true(true, "True is true");
});

// We must define at least one test for the harness, though it is not what we
// are testing here.
test(function() {
    assert_false(false, "False is false");
}, 'Worker test');
