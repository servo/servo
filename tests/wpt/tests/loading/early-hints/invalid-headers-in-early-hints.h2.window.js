async function testInvalidHeader(t, header_value) {
    const params = new URLSearchParams();
    params.set("header-value", header_value);
    const test_url = "resources/invalid-headers-in-early-hints.h2.py?" + params.toString();
    const opened_window = window.open(test_url, "invalid-header-in-early-hints");

    // Use step_timeout() because neither "load" event nor postMessage() would
    // work. Opening the test page should result in a network protocol error and
    // accessing the document of the opened window should throw a SecurityError.
    await new Promise(resolve => t.step_timeout(resolve, 1000));
    assert_throws_dom("SecurityError", () => {
        opened_window.document;
    }, "window.open() should not load the test page successfully.");
}

promise_test(async (t) => {
    await testInvalidHeader(t, "foo\r\nbar");
}, "Early Hints contains invalid header: newline byte");

promise_test(async (t) => {
    await testInvalidHeader(t, "foo\x00bar");
}, "Early Hints contains invalid header: nul byte");
