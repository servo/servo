// This tests the issues discussed in https://github.com/whatwg/html/issues/4299
// and fixed in https://github.com/whatwg/html/pull/6567.

// Note: because browsers do not interoperate on the spec's notion of window reuse (see e.g. https://crbug.com/778318)
// we pick a specific interoperable test case, which is "currently on initial about:blank, but loading something".

async_test(t => {
  const iframe = document.createElement("iframe");

  // We can't just leave it at the actual initial about:blank because of the interop issues mentioned above.
  // So put it in the "currently on initial about:blank, but loading something" state which interoperably does Window
  // reuse.
  iframe.src = "/common/blank.html";

  // Create the Window object. It will be for the initial about:blank since the load of /common/blank.html hasn't
  // completed.
  document.body.append(iframe);

  // Store a string on that Window object so we can later test if it's reused.
  iframe.contentWindow.persistedString = "Hello world!";

  // This will reset the initial about:blank-ness. But, it will also cancel any ongoing loads.
  iframe.contentDocument.open();

  // So, re-start the load of /common/blank.html.
  iframe.src = "/common/blank.html";

  // When the load finally happens, will it reuse the Window object or not?
  // Because document.open() resets the initial about:blank-ness, it will *not* reuse the Window object.
  // The point of the test is to assert that.
  iframe.addEventListener("load", t.step_func_done(() => {
    assert_equals(
      iframe.contentDocument.URL,
      iframe.src,
      "Prerequisite check: we are getting the right load event"
    );

    assert_equals(iframe.contentWindow.persistedString, undefined);
  }), { once: true });
}, "document.open() removes the initial about:blank-ness of the document");

// This test is redundant with others in WPT but it's intended to make it clear that document.open() is the
// distinguishing factor. It does the same exact thing but without document.open() and with the resulting final assert
// flipped.
async_test(t => {
  const iframe = document.createElement("iframe");
  iframe.src = "/common/blank.html";
  document.body.append(iframe);

  iframe.contentWindow.persistedString = "Hello world!";

  // NO document.open() call.

  iframe.src = "/common/blank.html";

  iframe.addEventListener("load", t.step_func_done(() => {
    assert_equals(
      iframe.contentDocument.URL,
      iframe.src,
      "Prerequisite check: we are getting the right load event"
    );

    assert_equals(iframe.contentWindow.persistedString, "Hello world!");
  }), { once: true });
}, "Double-check: without document.open(), Window reuse indeed happens");
