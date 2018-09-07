// The following tests deal with the <meta http-equiv=refresh> pragma and the
// `Refresh` header. The spec is still hazy on the precise behavior in those
// cases but we use https://github.com/whatwg/html/issues/4003 as a guideline.
//
// This is separate from abort-refresh-multisecond-meta.window.js to avoid
// browser interventions that limit the number of connections in a tab.

async_test(t => {
  const frame = document.body.appendChild(document.createElement("iframe"));
  t.add_cleanup(() => frame.remove());
  frame.onload = t.step_func(() => {
    frame.onload = null;
    let happened = false;

    const client = new frame.contentWindow.XMLHttpRequest();
    client.open("GET", "/common/blank.html");
    client.onload = t.step_func_done(() => {
      assert_true(happened);
    });
    client.onerror = t.unreached_func("XMLHttpRequest should have succeeded");
    client.onabort = t.unreached_func("XMLHttpRequest should have succeeded");
    client.ontimeout = t.unreached_func("XMLHttpRequest should have succeeded");
    client.send();

    frame.contentDocument.open();
    happened = true;
  });
  frame.src = "resources/http-refresh.py?1";
}, "document.open() does NOT abort documents that are queued for navigation through Refresh header with 1-sec timeout (XMLHttpRequest)");

async_test(t => {
  const frame = document.body.appendChild(document.createElement("iframe"));
  t.add_cleanup(() => frame.remove());
  frame.onload = t.step_func(() => {
    frame.onload = null;
    let happened = false;
    frame.contentWindow.fetch("/common/blank.html").then(
      t.step_func_done(() => {
        assert_true(happened);
      }),
      t.unreached_func("Fetch should have succeeded")
    );
    frame.contentDocument.open();
    happened = true;
  });
  frame.src = "resources/http-refresh.py?1";
}, "document.open() does NOT abort documents that are queued for navigation through Refresh header with 1-sec timeout (fetch())");

async_test(t => {
  const frame = document.body.appendChild(document.createElement("iframe"));
  t.add_cleanup(() => frame.remove());
  frame.onload = t.step_func(() => {
    frame.onload = null;
    let happened = false;
    const img = frame.contentDocument.createElement("img");
    img.src = new URL("resources/slow-png.py", document.URL);
    img.onload = t.step_func_done(() => {
      assert_true(happened);
    });
    img.onerror = t.unreached_func("Image loading should not have errored");
    // The image fetch starts in a microtask, so let's be sure to test after
    // the fetch has started.
    t.step_timeout(() => {
      frame.contentDocument.open();
      happened = true;
    });
  });
  frame.src = "resources/http-refresh.py?4";
}, "document.open() does NOT abort documents that are queued for navigation through Refresh header with 4-sec timeout (image loading)");
