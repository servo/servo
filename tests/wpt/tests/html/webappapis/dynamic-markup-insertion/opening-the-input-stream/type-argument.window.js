["replace",
 "NOBODY",
 "@ FD ;",
 "it does not matter, you see \f",
 "text/plain",
 "text/xml",
 "application/octet-stream",
 "\0"].forEach(type => {
  async_test(t => {
    const frame = document.body.appendChild(document.createElement("iframe"));
    t.add_cleanup(() => frame.remove());
    assert_equals(frame.contentDocument.open(type), frame.contentDocument);
    frame.contentDocument.write("<B>heya</b>");
    frame.contentDocument.close();
    assert_equals(frame.contentDocument.body.firstChild.localName, "b");
    assert_equals(frame.contentDocument.body.textContent, "heya");
    assert_equals(frame.contentDocument.contentType, "text/html");
    t.done();
  }, "document.open() with type set to: " + type + " (type argument is supposed to be ignored)");
});
