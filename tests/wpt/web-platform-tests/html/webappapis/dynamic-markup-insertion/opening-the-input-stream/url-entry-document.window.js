async_test(t => {
  const frame = document.body.appendChild(document.createElement("iframe"));
  t.add_cleanup(() => frame.remove());
  const frameURL = new URL("resources/url-entry-document-timer-frame.html", document.URL).href;
  window.timerTest = t.step_func_done(() => {
    assert_equals(frame.contentDocument.URL, frameURL);
    assert_equals(frame.contentWindow.location.href, frameURL);

    // In this case, the entry settings object was set when this function is
    // executed in the timer task through Web IDL's "invoke a callback
    // function" algorithm, to be the relevant settings object of this
    // function. Therefore the URL of this document would be inherited.
    assert_equals(frame.contentDocument.open(), frame.contentDocument);
    assert_equals(frame.contentDocument.URL, document.URL);
    assert_equals(frame.contentWindow.location.href, document.URL);
  });
  frame.src = frameURL;
}, "document.open() changes document's URL to the entry settings object's responsible document's (through timeouts)");
