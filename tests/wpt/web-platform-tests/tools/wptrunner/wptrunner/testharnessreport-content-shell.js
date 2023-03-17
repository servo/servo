(function() {
    var props = {output:%(output)d, debug: %(debug)s};
    setup(props);

    // Some tests navigate away from the original URL as part of the
    // functionality they exercise. In that case, `add_completion_callback(...)`
    // uses the final `window.location` to report the test ID, which may not be
    // correct [1].
    //
    // Persisting the original `window.location` with standard web platform APIs
    // (e.g., `localStorage`) could interfere with the their tests, so this must
    // be avoided. Unfortunately, there doesn't appear to be anything in content
    // shell's protocol mode or Blink-specific `window.testRunner` or
    // `window.internals` [2] that could help with this. As such, the driver
    // simply downgrades a mismatched test ID to a logged warning instead of a
    // harness error.
    //
    // [1] crbug.com/1418753
    // [2] https://chromium.googlesource.com/chromium/src/+/refs/heads/main/docs/testing/writing_web_tests.md#Relying-on-Blink_Specific-Testing-APIs
    const url = new URL(location.href);

    testRunner.dumpAsText();
    testRunner.waitUntilDone();
    testRunner.setPopupBlockingEnabled(false);
    testRunner.setDumpJavaScriptDialogs(false);
    // Show `CONSOLE MESSAGE:` and `CONSOLE ERROR:` in stderr.
    if (props.debug) {
        testRunner.setDumpConsoleMessages(true);
    }

    add_completion_callback(function (tests, harness_status) {
        const test_id = decodeURIComponent(url.pathname) + decodeURIComponent(url.search) + decodeURIComponent(url.hash);
        const result_string = JSON.stringify([
            test_id,
            harness_status.status,
            harness_status.message,
            harness_status.stack,
            tests.map(function(t) {
                return [t.name, t.status, t.message, t.stack]
            }),
        ]);
        testRunner.setCustomTextOutput(result_string);
        testRunner.notifyDone();
    });
})();
