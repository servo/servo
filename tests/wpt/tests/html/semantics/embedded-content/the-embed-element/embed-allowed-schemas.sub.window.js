async function test_uri(t, uri, pred) {
    const embed = document.createElement("embed");
    const promise = new Promise((resolve, reject) => {
        embed.onload = e => resolve(e.type);
        embed.src = uri;
        document.body.append(embed);
    });
    assert_equals(await promise, "load")
    assert_true(pred(embed));

    embed.remove();
}

// It's difficult detecting failure for <embed>, but the element
// having an offsetWidth of '0' is often enough.
function has_width(embed) {
    return embed.offsetWidth > 0;
}

promise_test(async t => {
    await test_uri(t, "about:blank", has_width);
}, "about: allowed in embed");

promise_test(async t => {
    const blobParts = ['Hello, world!'];
    const blob = new Blob(blobParts, { type: "text/html" })
    await test_uri(t, URL.createObjectURL(blob), has_width);
}, "blob: allowed in embed");

promise_test(async t => {
    await test_uri(t, "data:,Hello%2C%20World%21", has_width);
}, "data: allowed in embed");

promise_test(async t => {
    await test_uri(t, "https://{{domains[www]}}:{{ports[https][0]}}", has_width);
}, "https: allowed in embed");

promise_test(async t => {
    await test_uri(t, "http://{{domains[www]}}:{{ports[http][0]}}", has_width);
}, "http: allowed in embed");
