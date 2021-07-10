async_test(t => {
  const frame = document.body.appendChild(document.createElement("iframe"));
  t.add_cleanup(() => frame.remove());
  frame.onload = t.step_func(() => {
    frame.onload = null;
    const client = new frame.contentWindow.XMLHttpRequest();
    client.open("GET", "/common/blank.html");
    // The abort event handler is called synchronously in Chrome but
    // asynchronously in Firefox. See https://crbug.com/879620.
    client.onabort = t.step_func_done();
    client.send();
    frame.contentWindow.location.href = new URL("resources/dummy.html", document.URL);
    frame.contentDocument.open();
  });
  frame.src = "/common/blank.html";
}, "document.open() aborts documents that are navigating through Location (XMLHttpRequest)");

async_test(t => {
  const frame = document.body.appendChild(document.createElement("iframe"));
  t.add_cleanup(() => frame.remove());
  frame.onload = t.step_func(() => {
    frame.onload = null;
    let happened = false;
    frame.contentWindow.fetch("/common/blank.html").then(
      t.unreached_func("Fetch should have been aborted"),
      t.step_func_done(() => {
        assert_true(happened);
      }));
    frame.contentWindow.location.href = new URL("resources/dummy.html", document.URL);
    frame.contentDocument.open();
    happened = true;
  });
  frame.src = "/common/blank.html";
}, "document.open() aborts documents that are navigating through Location (fetch())");

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
      frame.contentWindow.location.href = new URL("resources/dummy.html", document.URL);
      frame.contentDocument.open();
      happened = true;
    });
    // If 3 seconds have passed and the image has still not loaded, we consider
    // it aborted. slow-png.py only sleeps for 2 wallclock seconds.
    t.step_timeout(t.step_func_done(() => {
      assert_true(happened);
    }), 3000);
  });
  frame.src = "/common/blank.html";
}, "document.open() aborts documents that are navigating through Location (image loading)");

async_test(t => {
  const div = document.body.appendChild(document.createElement("div"));
  t.add_cleanup(() => div.remove());
  div.innerHTML = "<iframe src='/common/slow.py'></iframe>";
  const frame = div.childNodes[0];
  const client = new frame.contentWindow.XMLHttpRequest();
  client.open("GET", "/common/blank.html");
  client.onabort = t.step_func_done();
  client.send();
  frame.contentDocument.open();
}, "document.open() aborts documents that are navigating through iframe loading (XMLHttpRequest)");

async_test(t => {
  const div = document.body.appendChild(document.createElement("div"));
  t.add_cleanup(() => div.remove());
  div.innerHTML = "<iframe src='/common/slow.py'></iframe>";
  const frame = div.childNodes[0];
  frame.contentWindow.fetch("/common/blank.html").then(
    t.unreached_func("Fetch should have been aborted"),
    t.step_func_done());
  frame.contentDocument.open();
}, "document.open() aborts documents that are navigating through iframe loading (fetch())");

// We cannot test for img element's error event for this test, as Firefox does
// not fire the event if the fetch is aborted while Chrome does.
//
// We use /common/slow.py here as the source of the iframe, to prevent the
// situation where when document.open() is called the initial about:blank
// document has already become inactive.
async_test(t => {
  const div = document.body.appendChild(document.createElement("div"));
  t.add_cleanup(() => div.remove());
  div.innerHTML = "<iframe src='/common/slow.py'></iframe>";
  const frame = div.childNodes[0];
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
}, "document.open() aborts documents that are navigating through iframe loading (image loading)");

async_test(t => {
  const frame = document.body.appendChild(document.createElement("iframe"));
  t.add_cleanup(() => frame.remove());
  frame.onload = t.step_func(() => {
    frame.onload = null;
    const link = frame.contentDocument.body.appendChild(frame.contentDocument.createElement("a"));
    link.href = new URL("resources/dummy.html", document.URL);

    const client = new frame.contentWindow.XMLHttpRequest();
    client.open("GET", "/common/blank.html");
    client.onabort = t.step_func_done();
    client.send();

    link.click();
    frame.contentDocument.open();
  });
  frame.src = "/common/blank.html";
}, "document.open() aborts documents that are queued for navigation through .click() (XMLHttpRequest)");

async_test(t => {
  const frame = document.body.appendChild(document.createElement("iframe"));
  t.add_cleanup(() => frame.remove());
  frame.onload = t.step_func(() => {
    frame.onload = null;
    const link = frame.contentDocument.body.appendChild(frame.contentDocument.createElement("a"));
    link.href = new URL("resources/dummy.html", document.URL);

    frame.contentWindow.fetch("/common/blank.html").then(
      t.unreached_func("Fetch should have been aborted"),
      t.step_func_done());

    link.click();
    frame.contentDocument.open();
  });
  frame.src = "/common/blank.html";
}, "document.open() aborts documents that are queued for navigation through .click() (fetch())");

// We cannot test for img element's error event for this test, as Firefox does
// not fire the event if the fetch is aborted while Chrome does.
async_test(t => {
  const frame = document.body.appendChild(document.createElement("iframe"));
  t.add_cleanup(() => frame.remove());
  frame.onload = t.step_func(() => {
    frame.onload = null;
    const link = frame.contentDocument.body.appendChild(frame.contentDocument.createElement("a"));
    link.href = new URL("resources/dummy.html", document.URL);

    let happened = false;
    const img = frame.contentDocument.createElement("img");
    img.src = new URL("resources/slow-png.py", document.URL);
    img.onload = t.unreached_func("Image loading should not have succeeded");
    // The image fetch starts in a microtask, so let's be sure to test after
    // the fetch has started.
    t.step_timeout(() => {
      link.click();
      frame.contentDocument.open();
      happened = true;
    });
    // If 3 seconds have passed and the image has still not loaded, we consider
    // it aborted. slow-png.py only sleeps for 2 wallclock seconds.
    t.step_timeout(t.step_func_done(() => {
      assert_true(happened);
    }), 3000);
  });
  frame.src = "/common/blank.html";
}, "document.open() aborts documents that are queued for navigation through .click() (image loading)");
