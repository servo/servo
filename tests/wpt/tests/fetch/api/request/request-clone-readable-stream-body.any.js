// META: global=window,worker

promise_test(async () => {
    const stream = new ReadableStream({
        start(controller) {
            controller.enqueue(new TextEncoder().encode("hello"));
            controller.close();
        }
    });

    const r1 = new Request("https://example.com/", { method: "POST", body: stream, duplex: "half" });
    const r2 = r1.clone();
    assert_false(r2.bodyUsed, "clone should not be marked as used");
    assert_not_equals(r2.body, null, "clone should have a body");

    // Constructing a new Request from the clone should preserve the body.
    const r3 = new Request(r2);
    assert_not_equals(r3.body, null, "Request constructed from clone should have a body");

    const text = await r3.text();
    assert_equals(text, "hello", "body content should be preserved through clone() and new Request()");
}, "new Request(clone) preserves a ReadableStream body that came from clone()");
