// META: global=window,dedicatedworker,sharedworker

promise_test(async test => {
    const result = await import("./export-hello.js", { with: { } });
    assert_equals(result.default, "hello");
}, "Dynamic import with an empty with clause should succeed");

promise_test(async test => {
    return promise_rejects_js(test, TypeError,
        import("./export-hello.js", { with: { unsupportedAssertionKey: "unsupportedAssertionValue"} }),
        "Dynamic import with an unsupported import attribute should fail");
}, "Dynamic import with an unsupported import attribute should fail");

promise_test(test => {
    return promise_rejects_js(test, TypeError,
        import("./export-hello.js", { with: { type: "notARealType"} } ),
        "Dynamic import with an unsupported type attribute should fail");
}, "Dynamic import with an unsupported type attribute should fail");
