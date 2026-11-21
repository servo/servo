async_test(t => {
  const blankDocumentURL = new URL("/common/blank.html", location).href;
  const frame = document.createElement("iframe");
  t.add_cleanup(() => frame.remove());
  frame.src = blankDocumentURL;
  document.body.append(frame);
  const initialRegistry = frame.contentWindow.customElements;
  frame.contentWindow.eval(`
    class AA extends HTMLElement { };
    self.globalAA = AA;
    customElements.define("a-a", AA);
    document.body.innerHTML = "<a-a>";
  `);
  assert_equals(frame.contentDocument.body.firstChild.localName, "a-a");
  assert_true(frame.contentDocument.body.firstChild instanceof frame.contentWindow.globalAA);

  frame.onload = t.step_func_done(t => {
    assert_equals(frame.contentDocument.URL, blankDocumentURL);
    assert_equals(frame.contentDocument.body.innerHTML, "");
    frame.contentDocument.body.innerHTML = "<a-a>";
    assert_equals(frame.contentDocument.body.firstChild.localName, "a-a");
    assert_not_equals(frame.contentWindow.globalAA, undefined, "The Window must be reused");
    assert_not_equals(frame.contentWindow.customElements, initialRegistry,
                      "The replacement document must get a new registry");
    assert_equals(frame.contentWindow.customElements.get("a-a"), undefined);
  });
}, "Each navigable document has its own registry");
