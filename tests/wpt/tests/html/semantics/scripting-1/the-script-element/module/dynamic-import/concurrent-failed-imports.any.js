// META: global=window,dedicatedworker,sharedworker
// META: script=/common/utils.js

// Additional coverage for "Don't cache HTTP errors in the module map"
// (https://github.com/whatwg/html/pull/10327), complementing the tests in
// repeated-imports.any.js.
//
// These focus on the concurrency guarantees of the module map's
// "list of callbacks" mechanism, which the spec change relies on:
//  1. Concurrent imports of a failing module are joined into a single fetch,
//     and every joined import rejects.
//  2. Each joined import is notified with the result of *its own* fetch, even
//     if a sibling import's rejection handler synchronously starts a new import
//     of the same specifier (which, because failures are not cached, performs a
//     fresh fetch). A joined import must never observe the internal
//     placeholder/list value nor another fetch's result.

promise_test(async test => {
    const uuid_token = token();
    // module-fetch-tester.py with mode=always-fail responds with an HTTP 404 to
    // every request, and counts how many requests reached the server.
    const url = `../../module-fetch-tester.py?key=${uuid_token}&mode=always-fail`;

    // Start several imports of the same specifier concurrently. They should be
    // joined into a single fetch via the module map, and all must reject.
    const importPromises = [];
    for (let i = 0; i < 5; i++) {
        importPromises.push(import(`${url}`));
    }
    await Promise.all(importPromises.map((p, i) =>
        promise_rejects_js(test, TypeError, p,
            `concurrent import #${i + 1} of a failing module must reject`)));

    // Despite five concurrent imports, the module must only have been fetched
    // once, because they were joined into a single in-flight fetch.
    const request_counter = await fetch(
        `../../module-fetch-tester.py?key=${uuid_token}&action=stat`, { cache: 'no-cache' });
    assert_equals(await request_counter.text(), "1",
        "concurrent imports of a failing module should be joined into a single fetch");
}, "Multiple concurrent imports of a failing module should all reject but only fetch once");

promise_test(async test => {
    const uuid_token = token();
    // module-fetch-tester.py (default mode=fail-first) fails the first request
    // for a given key and succeeds on every subsequent request, so import #3 can
    // be started *synchronously* from import #1's rejection handler (while
    // draining the microtask queue), with no intervening configuration
    // fetch/task. This reproduces the module map callback race discussed in
    // whatwg/html#10327.
    const url = `../../module-fetch-tester.py?key=${uuid_token}`;

    // import #1 and import #2 are started in parallel, so they are joined into a
    // single fetch. That fetch is the first request for this key, so it fails:
    //  - Both import #1 and import #2 must reject with the result of that fetch.
    //  - import #1's rejection handler synchronously re-imports the same
    //    specifier (import #3). Because the failed fetch was not cached, import
    //    #3 performs a fresh fetch, which (being the second request) succeeds.
    //  - import #2 must still reject: it is tied to the first (failed) fetch and
    //    must not observe the module map's list placeholder nor import #3's
    //    successful result.
    const import1Promise = import(`${url}`);
    const import2Promise = import(`${url}`);

    // import #2 must reject with a TypeError from the first, failed fetch.
    const import2Check = promise_rejects_js(test, TypeError, import2Promise,
        "import #2 must reject with the first (failed) fetch result, not resolve " +
        "with import #3's success");

    // import #1 rejects; from its rejection handler we re-import the same
    // specifier (import #3) synchronously, which must succeed on the second request.
    const import1Check = import1Promise.then(
        () => assert_unreached("import #1 must not resolve; the first fetch failed"),
        () => import(`${url}`).then(ns => { // import #3
            assert_equals(ns.default, "hello",
                "import #3 should perform a fresh fetch and succeed, since the " +
                "failed fetch was not cached");
        }));

    await Promise.all([import1Check, import2Check]);
}, "A joined import is notified with its own fetch result even if a sibling's " +
   "rejection handler re-imports the same specifier");
