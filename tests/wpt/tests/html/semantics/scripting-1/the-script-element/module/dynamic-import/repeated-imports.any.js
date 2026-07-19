// META: global=window,dedicatedworker,sharedworker
// META: script=/common/utils.js

promise_test(async test => {
    const uuid_token = token();
    // Set up the server to respond with JSON first
    await fetch(`../../serve-custom-response.py?key=${uuid_token}&action=set&content-type=application/json`, { cache: 'no-cache' });
    await promise_rejects_js(test, TypeError,
        import(`../../serve-custom-response.py?key=${uuid_token}`),
      "Dynamic import of JS with a JSON type response should fail");
    let request_counter = await fetch(`../../serve-custom-response.py?key=${uuid_token}&action=stat`, { cache: 'no-cache' })
    assert_equals(await request_counter.text(), "1");

    // Import using the same specifier/type pair again; this time server still responds with JSON
    await promise_rejects_js(test, TypeError,
        import(`../../serve-custom-response.py?key=${uuid_token}`),
        "Dynamic import of JS with a JSON type response should fail again");
    // the server should have been contacted again because the failed import attempt did not create a module map entry
    request_counter = await fetch(`../../serve-custom-response.py?key=${uuid_token}&action=stat`, { cache: 'no-cache' })
    assert_equals(await request_counter.text(), "2");

    // Import using the same specifier/type pair again; this time we get JS
    await fetch(`../../serve-custom-response.py?key=${uuid_token}&action=set&content-type=application/javascript`, { cache: 'no-cache' });
    // it should succeed since the failed import attempt did not create a module map entry
    const result_js = await import(`../../serve-custom-response.py?key=${uuid_token}`);
    assert_equals(result_js.default, "hello");
    request_counter = await fetch(`../../serve-custom-response.py?key=${uuid_token}&action=stat`, { cache: 'no-cache' })
    assert_equals(await request_counter.text(), "3");

    // Import using the same specifier/type pair again; this time server should not be contacted
    // because the module map entry should already exist.
    const result_js_2 = await import(`../../serve-custom-response.py?key=${uuid_token}`);
    assert_equals(result_js_2.default, "hello");
    request_counter = await fetch(`../../serve-custom-response.py?key=${uuid_token}&action=stat`, { cache: 'no-cache' })
    assert_equals(await request_counter.text(), "3");
}, "An import should succeed even if the same specifier/type attribute pair previously failed due to a MIME type mismatch");

promise_test(async test => {
    const uuid_token = token();
    // Set up the server to respond with network error first
    await fetch(`../../serve-custom-response.py?key=${uuid_token}&action=set&network-error=true`, { cache: 'no-cache' });
    await promise_rejects_js(test, TypeError,
        import(`../../serve-custom-response.py?key=${uuid_token}`),
        "Dynamic import should fail with a network error");
    let request_counter = await fetch(`../../serve-custom-response.py?key=${uuid_token}&action=stat`, { cache: 'no-cache' })
    // some browsers may request multiple times on network error
    assert_not_equals(await request_counter.text(), "0");
    await fetch(`../../serve-custom-response.py?key=${uuid_token}&action=reset-stat`, { cache: 'no-cache' });

    // Then set up the server to respond with 404
    await fetch(`../../serve-custom-response.py?key=${uuid_token}&action=set&network-error=false&code=404`, { cache: 'no-cache' });
    await promise_rejects_js(test, TypeError,
        import(`../../serve-custom-response.py?key=${uuid_token}`),
        "Dynamic import should fail with a 404 response");
    request_counter = await fetch(`../../serve-custom-response.py?key=${uuid_token}&action=stat`, { cache: 'no-cache' })
    assert_equals(await request_counter.text(), "1");

    // Import using the same specifier again; this time server still responds with 404
    await promise_rejects_js(test, TypeError,
        import(`../../serve-custom-response.py?key=${uuid_token}`),
        "Dynamic import should fail again with a 404 response");
    request_counter = await fetch(`../../serve-custom-response.py?key=${uuid_token}&action=stat`, { cache: 'no-cache' })
    // the server should have been contacted again because the failed import attempt did not create a module map entry
    assert_equals(await request_counter.text(), "2");

    // Import using the same specifier again; this time we configure the server to respond with 200
    await fetch(`../../serve-custom-response.py?key=${uuid_token}&action=set&code=200`, { cache: 'no-cache' });
    // it should succeed since the failed import attempt did not create a module map entry
    const result_js = await import(`../../serve-custom-response.py?key=${uuid_token}`);
    assert_equals(result_js.default, "hello");
    request_counter = await fetch(`../../serve-custom-response.py?key=${uuid_token}&action=stat`, { cache: 'no-cache' })
    assert_equals(await request_counter.text(), "3");

    // Import using the same specifier again; this time server should not be contacted
    // because the module map entry should already exist.
    const result_js_2 = await import(`../../serve-custom-response.py?key=${uuid_token}`);
    assert_equals(result_js_2.default, "hello");
    request_counter = await fetch(`../../serve-custom-response.py?key=${uuid_token}&action=stat`, { cache: 'no-cache' })
    assert_equals(await request_counter.text(), "3");
}, "An import should succeed even if the same specifier/type pair previously failed due to a network error (e.g. 404)");

promise_test(async test => {
    const uuid_token = token();
    await fetch(`../../serve-custom-response.py?key=${uuid_token}&action=set&code=200`, { cache: 'no-cache' });
    for (let i = 0; i < 5; i++) {
        const result_js = await import(`../../serve-custom-response.py?key=${uuid_token}`);
        assert_equals(result_js.default, "hello",
          `JS module should load successfully on attempt ${i + 1}`);
    }
    const request_counter = await fetch(`../../serve-custom-response.py?key=${uuid_token}&action=stat`, { cache: 'no-cache' });
    const statText = await request_counter.text();
    assert_equals(statText, '1', 'Import should only be fetched once for multiple imports');
}, "Multiple imports in sequence with the same specifier/type pair should only fetch the module once");

promise_test(async test => {
    const uuid_token = token();
    await fetch(`../../serve-custom-response.py?key=${uuid_token}&action=set&code=200`, { cache: 'no-cache' });
    const importPromises = [];
    for (let i = 0; i < 5; i++) {
        importPromises.push(import(`../../serve-custom-response.py?key=${uuid_token}`));
    }
    const results = await Promise.all(importPromises);
    results.forEach((result_js, index) => {
        assert_equals(result_js.default, "hello",
          `JS module should load successfully on attempt ${index + 1}`);
    });
    const request_counter = await fetch(`../../serve-custom-response.py?key=${uuid_token}&action=stat`, { cache: 'no-cache' });
    const statText = await request_counter.text();
    assert_equals(statText, '1', 'Import should only be fetched once for multiple concurrent imports');
}, "Multiple concurrent imports with the same specifier/type pair should only fetch the module once");
