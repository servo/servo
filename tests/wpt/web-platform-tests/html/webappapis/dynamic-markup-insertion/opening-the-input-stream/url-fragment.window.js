async_test(t => {
  const frame = document.body.appendChild(document.createElement("iframe")),
        urlSansHash = document.URL;
  t.add_cleanup(() => { frame.remove(); });
  assert_equals(frame.contentDocument.URL, "about:blank");
  assert_equals(frame.contentWindow.location.href, "about:blank");
  self.onhashchange = t.step_func_done(() => {
    frame.contentDocument.open();
    assert_equals(frame.contentDocument.URL, urlSansHash);
    assert_equals(frame.contentWindow.location.href, urlSansHash);
  });
  self.location.hash = "heya";
}, "document.open() and document's URL containing a fragment (entry is not relevant)");

window.testDone = undefined;
async_test(t => {
  const frame = document.body.appendChild(document.createElement("iframe"))
  t.add_cleanup(() => { frame.remove(); });
  frame.src = "resources/url-frame.html#heya";
  window.testDone = t.step_func_done((beforeURL, afterURL) => {
    assert_equals(beforeURL, frame.src);
    assert_equals(afterURL, frame.src);
    assert_equals(frame.contentDocument.URL, frame.src);
    assert_equals(frame.contentWindow.location.href, frame.src);
  });
}, "document.open() and document's URL containing a fragment (entry is relevant)");
