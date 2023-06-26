// META: script=/common/utils.js
// META: script=resources/early-hints-helpers.sub.js

promise_test(async (t) => {
    const resource_url = SAME_ORIGIN_RESOURCES_URL + "/empty.js?" + token();
    const promise = new Promise((resolve) => {
        const params = new URLSearchParams();
        params.set("resource-url", resource_url);
        params.set("token", token());
        const embed_url = SAME_ORIGIN_RESOURCES_URL + "/png-with-early-hints.h2.py?" + params.toString();

        const el = document.createElement("embed");
        el.src = embed_url;
        el.onload = resolve;
        document.body.appendChild(el);
    });
    await promise;

    await fetchScript(resource_url);
    const entries = performance.getEntriesByName(resource_url);
    assert_equals(entries.length, 1);
    assert_not_equals(entries[0].transferSize, 0);
}, "Early hints for an embed element should be ignored.");

promise_test(async (t) => {
    const resource_url = SAME_ORIGIN_RESOURCES_URL + "/empty.js?" + token();
    const promise = new Promise((resolve) => {
        const params = new URLSearchParams();
        params.set("resource-url", resource_url);
        params.set("token", token());
        const object_url = SAME_ORIGIN_RESOURCES_URL + "/png-with-early-hints.h2.py?" + params.toString();

        const el = document.createElement("object");
        el.data = object_url;
        el.onload = resolve;
        document.body.appendChild(el);
    });
    await promise;

    await fetchScript(resource_url);
    const entries = performance.getEntriesByName(resource_url);
    assert_equals(entries.length, 1);
    assert_not_equals(entries[0].transferSize, 0);
}, "Early hints for an object element should be ignored.");
