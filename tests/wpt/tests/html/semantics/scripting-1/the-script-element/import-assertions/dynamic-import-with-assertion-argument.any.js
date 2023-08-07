// META: global=window,dedicatedworker,sharedworker

promise_test(async test => {
    const result = await import("./export-hello.js", { assert: { } });
    assert_equals(result.default, "hello");
}, "Dynamic import with an empty assert clause should succeed");

promise_test(async test => {
    return promise_rejects_js(test, TypeError,
        import("./export-hello.js", { assert: { unsupportedAssertionKey: "unsupportedAssertionValue"} }),
        "Dynamic import with an unsupported import assertion should fail");
}, "Dynamic import with an unsupported import assertion should fail");

promise_test(test => {
    return promise_rejects_js(test, TypeError,
        import("./export-hello.js", { assert: { type: "notARealType"} } ),
        "Dynamic import with an unsupported type assertion should fail");
}, "Dynamic import with an unsupported type assertion should fail");
