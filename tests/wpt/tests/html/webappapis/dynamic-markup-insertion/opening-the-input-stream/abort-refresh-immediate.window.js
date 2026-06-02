// The following tests deal with the <meta http-equiv=refresh> pragma and the
// `Refresh` header. The spec is still hazy on the precise behavior in those
// cases but we use https://github.com/whatwg/html/issues/4003 as a guideline.

async_test(t => {
  const frame = document.body.appendChild(document.createElement("iframe"));
  t.add_cleanup(() => frame.remove());
  frame.onload = t.step_func(() => {
    frame.onload = null;

    const client = new frame.contentWindow.XMLHttpRequest();
    client.open("GET", "/common/blank.html");
    client.onabort = t.step_func_done();
    client.send();

    frame.contentDocument.open();
  });
  frame.src = "resources/meta-refresh.py?0";
}, "document.open() aborts documents that are queued for navigation through <meta> refresh with timeout 0 (XMLHttpRequest)");

async_test(t => {
  const frame = document.body.appendChild(document.createElement("iframe"));
  t.add_cleanup(() => frame.remove());
  frame.onload = t.step_func(() => {
    frame.onload = null;

    frame.contentWindow.fetch("/common/blank.html").then(
      t.unreached_func("Fetch should have been aborted"),
      t.step_func_done());

    frame.contentDocument.open();
  });
  frame.src = "resources/meta-refresh.py?0";
}, "document.open() aborts documents that are queued for navigation through <meta> refresh with timeout 0 (fetch())");

// We cannot test for img element's error event for this test, as Firefox does
// not fire the event if the fetch is aborted while Chrome does.
async_test(t => {
  const frame = document.body.appendChild(document.createElement("iframe"));
  t.add_cleanup(() => frame.remove());
  frame.onload = t.step_func(() => {
    frame.onload = null;

    let happened = false;
    const img = frame.contentDocument.createElement("img");
    img.src = new URL("resources/slow-png.py", document.URL);
    img.onload = t.unreached_func("Image loading should not have succeeded");
    // The image fetch starts in a microtask, so let's be sure to test after
    // the fetch has started.
    t.step_timeout(() => {
      frame.contentDocument.open();
      happened = true;
    });
    // If 3 seconds have passed and the image has still not loaded, we consider
    // it aborted. slow-png.py only sleeps for 2 wallclock seconds.
    t.step_timeout(t.step_func_done(() => {
      assert_true(happened);
    }), 3000);
  });
  frame.src = "resources/meta-refresh.py?0";
}, "document.open() aborts documents that are queued for navigation through <meta> refresh with timeout 0 (image loading)");

async_test(t => {
  const frame = document.body.appendChild(document.createElement("iframe"));
  t.add_cleanup(() => frame.remove());
  frame.onload = t.step_func(() => {
    frame.onload = null;

    const client = new frame.contentWindow.XMLHttpRequest();
    client.open("GET", "/common/blank.html");
    client.onabort = t.step_func_done();
    client.send();

    frame.contentDocument.open();
  });
  frame.src = "resources/http-refresh.py?0";
}, "document.open() aborts documents that are queued for navigation through Refresh header with timeout 0 (XMLHttpRequest)");

async_test(t => {
  const frame = document.body.appendChild(document.createElement("iframe"));
  t.add_cleanup(() => frame.remove());
  frame.onload = t.step_func(() => {
    frame.onload = null;

    frame.contentWindow.fetch("/common/blank.html").then(
      t.unreached_func("Fetch should have been aborted"),
      t.step_func_done());

    frame.contentDocument.open();
  });
  frame.src = "resources/http-refresh.py?0";
}, "document.open() aborts documents that are queued for navigation through Refresh header with timeout 0 (fetch())");

// We cannot test for img element's error event for this test, as Firefox does
// not fire the event if the fetch is aborted while Chrome does.
async_test(t => {
  const frame = document.body.appendChild(document.createElement("iframe"));
  t.add_cleanup(() => frame.remove());
  frame.onload = t.step_func(() => {
    frame.onload = null;

    let happened = false;
    const img = frame.contentDocument.createElement("img");
    img.src = new URL("resources/slow-png.py", document.URL);
    img.onload = t.unreached_func("Image loading should not have succeeded");
    // The image fetch starts in a microtask, so let's be sure to test after
    // the fetch has started.
    t.step_timeout(() => {
      frame.contentDocument.open();
      happened = true;
    });
    // If 3 seconds have passed and the image has still not loaded, we consider
    // it aborted. slow-png.py only sleeps for 2 wallclock seconds.
    t.step_timeout(t.step_func_done(() => {
      assert_true(happened);
    }), 3000);
  });
  frame.src = "resources/http-refresh.py?0";
}, "document.open() aborts documents that are queued for navigation through Refresh header with timeout 0 (image loading)");
