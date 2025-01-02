async_test(t => {
  const frame = document.body.appendChild(document.createElement("iframe"));
  t.add_cleanup(() => { frame.remove(); });
  const originalHTMLElement = frame.contentDocument.documentElement;
  assert_equals(originalHTMLElement.localName, "html");
  const observer = new frame.contentWindow.MutationObserver(t.step_func_done(records => {
    // Even though we passed `subtree: true` to observer.observe, due to the
    // fact that "replace all" algorithm removes children with the "suppress
    // observers flag" set, we still only get the html element as the sole
    // removed node.
    assert_equals(records.length, 1);
    assert_equals(records[0].type, "childList");
    assert_equals(records[0].target, frame.contentDocument);
    assert_array_equals(records[0].addedNodes, []);
    assert_array_equals(records[0].removedNodes, [originalHTMLElement]);
  }));
  observer.observe(frame.contentDocument, { childList: true, subtree: true });
  assert_equals(frame.contentDocument.open(), frame.contentDocument);
}, "document.open() should inform mutation observer of node removal");
