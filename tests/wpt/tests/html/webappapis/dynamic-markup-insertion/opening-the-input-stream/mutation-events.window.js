// In an ideal world this test would eventually be obsolete due to mutation events disappearing. Or
// would have to change to account for mutation events not firing synchronously. Neither seems
// realistic to the author though.

test(t => {
  const frame = document.body.appendChild(document.createElement("iframe"));
  frame.contentWindow.addEventListener("DOMNodeInserted", t.unreached_func());
  frame.contentWindow.addEventListener("DOMNodeInserted", t.unreached_func(), true);
  frame.contentWindow.addEventListener("DOMNodeInsertedIntoDocument", t.unreached_func(), true);
  frame.contentWindow.addEventListener("DOMNodeRemoved", t.unreached_func());
  frame.contentWindow.addEventListener("DOMNodeRemoved", t.unreached_func(), true);
  frame.contentWindow.addEventListener("DOMNodeRemovedFromDocument", t.unreached_func(), true);
  frame.contentWindow.addEventListener("DOMSubtreeModified", t.unreached_func());
  frame.contentWindow.addEventListener("DOMSubtreeModified", t.unreached_func(), true);
  assert_equals(frame.contentDocument.documentElement.localName, "html");
  assert_equals(frame.contentDocument.open(), frame.contentDocument);
  assert_equals(frame.contentDocument.documentElement, null);
  frame.contentDocument.write("<div>heya</div>");
  frame.contentDocument.close();
  assert_equals(frame.contentDocument.documentElement.localName, "html");
  frame.remove();
}, "document.open(), the HTML parser, and mutation events");
