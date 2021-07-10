importScripts("/resources/testharness.js");

setup({ single_test: true });

// Because this script enables the `single_test` configuration option, it
// should be interpreted as a single-page test, and the uncaught exception
// should be reported as a test failure (harness status: OK).
throw new Error("This failure is expected.");
