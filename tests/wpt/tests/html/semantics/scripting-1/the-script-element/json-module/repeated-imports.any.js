// META: global=window,dedicatedworker,sharedworker
// META: script=/common/utils.js

promise_test(async test => {
    await promise_rejects_js(test, TypeError,
      import("./module.json"),
      "Dynamic import of a JSON module without a type assertion should fail");

    // This time the import should succeed because we're using the correct
    // import even though the previous attempt with the same  specifier failed.
    const result = await import("./module.json", { assert: { type: "json" } });
    assert_true(result.default.test);
}, "Importing a specifier that previously failed due to an incorrect type assertion can succeed if the correct assertion is later given");

promise_test(async test => {
    // Append a URL fragment to the specifier so that this is independent
    // from the previous test.
    const result = await import("./module.json#2", { assert: { type: "json" } });
    assert_true(result.default.test);

    await promise_rejects_js(test, TypeError,
      import("./module.json#2"),
      "Dynamic import should fail with the type assertion missing even if the same specifier previously succeeded");
}, "Importing a specifier that previously succeeded with the correct type assertion should fail if the incorrect assertion is later given");

promise_test(async test => {
    const uuid_token = token();
    // serve-json-then-js.py gives us JSON the first time
    const result_json = await import(`../serve-json-then-js.py?key=${uuid_token}`, { assert: { type: "json" } });
    assert_equals(result_json.default.hello, "world");

    // Import using the same specifier again; this time we get JS, which
    // should succeed since we're not asserting a non-JS type this time.
    const result_js = await import(`../serve-json-then-js.py?key=${uuid_token}`);
    assert_equals(result_js.default, "hello");
}, "Two modules of different type with the same specifier can load if the server changes its responses");

promise_test(async test => {
    const uuid_token = token();
    // serve-json-then-js.py gives us JSON the first time
    await promise_rejects_js(test, TypeError,
      import(`../serve-json-then-js.py?key=${uuid_token}`),
      "Dynamic import of JS with a JSON type assertion should fail");

    // Import using the same specifier/module type pair again; this time we get JS,
    // but the import should still fail because the module map entry for this
    // specifier/module type pair already contains a failure.
    await promise_rejects_js(test, TypeError,
      import(`../serve-json-then-js.py?key=${uuid_token}`),
      "import should always fail if the same specifier/type assertion pair failed previously");
}, "An import should always fail if the same specifier/type assertion pair failed previously");

promise_test(async test => {
    const uuid_token = token();
    // serve-json-then-js.py gives us JSON the first time
    const result_json = await import(`../serve-json-then-js.py?key=${uuid_token}`, { assert: { type: "json" } });
    assert_equals(result_json.default.hello, "world");

    // If this were to do another fetch, the import would fail because
    // serve-json-then-js.py would give us JS this time. But, the module map
    // entry for this specifier/module type pair already exists, so we
    // successfully reuse the entry instead of fetching again.
    const result_json_2 = await import(`../serve-json-then-js.py?key=${uuid_token}`, { assert: { type: "json" } });
    assert_equals(result_json_2.default.hello, "world");
}, "If an import previously succeeded for a given specifier/type assertion pair, future uses of that pair should yield the same result");
