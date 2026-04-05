// META: global=window,dedicatedworker,sharedworker
// META: script=/common/utils.js

promise_test(async test => {
    await promise_rejects_js(test, TypeError,
      import("./file.txt"),
      "Dynamic import of a text module without a type attribute should fail");

    // This time the import should succeed because we're using the correct
    // import even though the previous attempt with the same  specifier failed.
    const result = await import("./file.txt", { with: { type: "text" } });
    assert_equals(result.default, "text file\n");
}, "Importing a specifier that previously failed due to an incorrect type attribute can succeed if the correct attribute is later given");

promise_test(async test => {
    // Append a URL fragment to the specifier so that this is independent
    // from the previous test.
    const result = await import("./file.txt#2", { with: { type: "text" } });
    assert_equals(result.default, "text file\n");

    await promise_rejects_js(test, TypeError,
      import("./file.txt#2"),
      "Dynamic import should fail with the type attribute missing even if the same specifier previously succeeded");
}, "Importing a specifier that previously succeeded with the correct type attribute should fail if the incorrect attribute is later given");

promise_test(async test => {
    const uuid_token = token();
    // serve-text-then-js.py gives us text the first time
    const result_text = await import(`./serve-text-then-js.py?key=${uuid_token}`, { with: { type: "text" } });
    assert_equals(result_text.default, "hello");

    // Import using the same specifier again; this time we get JS, which
    // should succeed since we're not asserting a non-JS type this time.
    const result_js = await import(`./serve-text-then-js.py?key=${uuid_token}`);
    assert_equals(result_js.default, "world");
}, "Two modules of different type with the same specifier can load if the server changes its responses (first text, then JS)");

promise_test(async test => {
    const uuid_token = token();
    // serve-text-then-js.py gives us text the first time
    await promise_rejects_js(test, TypeError,
      import(`./serve-text-then-js.py?key=${uuid_token}`),
      "Dynamic import of text without a type attribute should fail");

    // Import using the same specifier/module type pair again; this time we get JS,
    // but the import should still fail because the module map entry for this
    // specifier/module type pair already contains a failure.
    await promise_rejects_js(test, TypeError,
      import(`./serve-text-then-js.py?key=${uuid_token}`),
      "import should always fail if the same specifier/type attribute pair failed previously");
}, "An import should always fail if the same specifier/type attribute pair failed previously");

promise_test(async test => {
    const uuid_token = token();
    // serve-text-then-js.py gives us text the first time
    const result_text = await import(`./serve-text-then-js.py?key=${uuid_token}`, { with: { type: "text" } });
    assert_equals(result_text.default, "hello");

    // If this were to do another fetch, the import would fail because
    // serve-text-then-js.py would give us JS this time. But, the module map
    // entry for this specifier/module type pair already exists, so we
    // successfully reuse the entry instead of fetching again.
    const result_text_2 = await import(`./serve-text-then-js.py?key=${uuid_token}`, { with: { type: "text" } });
    assert_equals(result_text_2.default, "hello");
}, "If an import previously succeeded for a given specifier/type attribute pair, future uses of that pair should yield the same result");

promise_test(async test => {
    const uuid_token = token();
    // serve-js-then-text.py gives us JS the first time
    const result_js = await import(`./serve-js-then-text.py?key=${uuid_token}`);
    assert_equals(result_js.default, "world");

    // Import using the same specifier again; this time we get text.
    const result_text = await import(`./serve-js-then-text.py?key=${uuid_token}`, { with: { type: "text" } });
    assert_equals(result_text.default, "hello");
}, "Two modules of different type with the same specifier can load if the server changes its responses (first JS, then text)");

promise_test(async test => {
    const uuid_token = token();
    // serve-js-then-text.py gives us JS the first time
    const result_js = await import(`./serve-js-then-text.py?key=${uuid_token}`);
    assert_equals(result_js.default, "hello");

    // If this were to do another fetch, the import would fail because
    // serve-js-then-text.py would give us text this time. But, the module map
    // entry for this specifier/module type pair already exists, so we
    // successfully reuse the entry instead of fetching again.
    const result_js_2 = await import(`./serve-js-then-text.py?key=${uuid_token}`);
    assert_equals(result_js_2.default, "hello");
}, "If an import previously succeeded for a given specifier with no type attribute, future uses of the same values should yield the same result");
