["replace",
 "NOBODY",
 "@ FD ;",
 "it does not matter, you see \f",
 "text/plain",
 "text/xml",
 "application/octet-stream",
 "\0"].forEach(type => {
  async_test(t => {
    const frame = document.createElement("iframe");
    frame.src = "type-argument-plaintext-subframe.txt";
    document.body.appendChild(frame);
    t.add_cleanup(() => frame.remove());
    frame.onload = t.step_func_done(() => {
      assert_equals(frame.contentDocument.open(type), frame.contentDocument);
      frame.contentDocument.write("<B>heya</b>");
      frame.contentDocument.close();
      assert_equals(frame.contentDocument.body.firstChild.localName, "b");
      assert_equals(frame.contentDocument.body.textContent, "heya");
      assert_equals(frame.contentDocument.contentType, "text/plain");
    });
  }, "document.open() on plaintext document with type set to: " + type + " (type argument is supposed to be ignored)");
});
