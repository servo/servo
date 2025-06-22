async function test_uri(t, uri, expected) {
    const object = document.createElement("object");
    const promise = new Promise((resolve, reject) => {
        object.onerror = e => reject(e.type);
        object.onload = () => resolve("success");
        object.data = uri;
        document.body.append(object);
    });

    if (expected === "success") {
        await promise;
    } else {
        await promise_rejects_exactly(t, expected, promise);
    }

    object.remove();
}

promise_test(async t => {
    await test_uri(t, "about:blank", "success");
}, "about: allowed in object");

promise_test(async t => {
    const blobParts = ['Hello, world!'];
    const blob = new Blob(blobParts, { type: "text/html" })
    await test_uri(t, URL.createObjectURL(blob), "success");
}, "blob: allowed in object");

promise_test(async t => {
    await test_uri(t, "data:,Hello%2C%20World%21", "success");
}, "data: allowed in object");

promise_test(async t => {
    await test_uri(t, "https://{{domains[www]}}:{{ports[https][0]}}", "success");
}, "https: allowed in object");

promise_test(async t => {
    await test_uri(t, "http://{{domains[www]}}:{{ports[http][0]}}", "success");
}, "http: allowed in object");


promise_test(async t => {
    await test_uri(t, "javascript:'x'", "error");
}, "javascript: scheme not allowed in object");
