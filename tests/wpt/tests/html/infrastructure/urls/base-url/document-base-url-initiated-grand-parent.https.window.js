// META: script=/common/get-host-info.sub.js
// META: script=/common/utils.js
// META: script=/common/dispatcher/dispatcher.js

const testBaseUriAboutBlankFromGrandParent = (description, child_origin) => {
  promise_test(async test => {
    // Create a child in an iframe.
    const child_token = token();
    const child_url = child_origin +
      '/common/dispatcher/executor.html' +
      `?uuid=${child_token}`;
    const iframe = document.createElement("iframe");
    iframe.src = child_url;
    document.body.appendChild(iframe);

    // The child creates a grand child in an iframe.
    const reply_token = token();
    send(child_token, `
      const iframe = document.createElement("iframe");
      location.hash = "interesting-fragment";
      iframe.src = "/common/blank.html";
      iframe.onload = () => {
        send("${reply_token}", "grand child loaded");
      };
      document.body.appendChild(iframe);
    `);
    assert_equals(await receive(reply_token), "grand child loaded");

    const child = iframe.contentWindow;
    const grandchild = child[0];

    // Navigate the grand-child toward about:blank.
    // Navigation are always asynchronous. It doesn't exist a ways to know the
    // about:blank document committed. A timer is used instead:
    grandchild.location = "about:blank";
    await new Promise(r => test.step_timeout(r, /*ms=*/500));

    // The grandchild baseURI must correspond to its grand parent.
    //
    // Note: `child_token` is removed, to get a stable failure, in case the
    // about:blank's document.baseURI reports the parent's URL instead of its
    // grand-parent.
    assert_equals(
        grandchild.document.baseURI.replace(child_token, "child_token"),
        self.document.baseURI);
  }, description);
}

onload = () => {
  testBaseUriAboutBlankFromGrandParent(
    "Check the baseURL of an about:blank document same-origin with its parent",
    get_host_info().HTTPS_ORIGIN,
  );
  testBaseUriAboutBlankFromGrandParent(
    "Check the baseURL of an about:blank document cross-origin with its parent",
    get_host_info().HTTPS_REMOTE_ORIGIN,
  );
  testBaseUriAboutBlankFromGrandParent(
    "Check the baseURL of an about:blank document cross-site with its parent",
    get_host_info().HTTPS_NOTSAMESITE_ORIGIN,
  );
}
