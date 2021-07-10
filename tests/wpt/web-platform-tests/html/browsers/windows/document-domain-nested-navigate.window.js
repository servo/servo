async_test(t => {
  // Setting document.domain makes this document cross-origin with that of the frame. However,
  // about:blank will end up reusing the origin of this document, at which point the frame's
  // document is no longer cross-origin.
  const frame = document.body.appendChild(document.createElement('iframe'));
  document.domain = document.domain;
  frame.src = "/common/blank.html";
  frame.onload = t.step_func(() => {
    assert_throws_dom("SecurityError", () => window[0].document);
    frame.src = "about:blank";
    frame.onload = t.step_func_done(() => {
      // Ensure we can access the child browsing context after navigation to non-initial about:blank
      assert_equals(window[0].document, frame.contentDocument);
    });
  });
}, "Navigated frame to about:blank and document.domain");
