var props = {output:%(output)d, debug: %(debug)s};
setup(props);

testRunner.dumpAsText();
testRunner.waitUntilDone();
testRunner.setPopupBlockingEnabled(false);
testRunner.setDumpJavaScriptDialogs(false);

add_completion_callback(function (tests, harness_status) {
    var id = decodeURIComponent(location.pathname) + decodeURIComponent(location.search) + decodeURIComponent(location.hash);
    var result_string = JSON.stringify([
        id,
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
