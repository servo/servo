// META: script=resources/util.js
// META: script=/common/utils.js

// The client hints attached to subresource requests are the ones that were
// enabled for the document when it was committed. So, after clearing any hints
// persisted by earlier tests, perform the fetch from a freshly loaded
// same-origin document rather than from this (possibly stale) test page.
make_subresource_test = (url) =>
  async (t) => {
    await clear_client_hints();
    const iframe = document.createElement("iframe");
    const loaded = new Promise((resolve) => iframe.onload = resolve);
    iframe.src = "/common/blank.html";
    document.body.appendChild(iframe);
    t.add_cleanup(() => iframe.remove());
    await loaded;
    const response = await iframe.contentWindow.fetch(new URL(url, location.href));
    const text = await response.text();
    assert_true(text.includes("FAIL"), "got: " + text);
  }

promise_test(make_subresource_test(ECHO_URL), "Critical-CH subresource fetch");

promise_test(make_subresource_test(ECHO_URL+"?multiple=true"), "Critical-CH w/ multiple headers and subresource fetch");
