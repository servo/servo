async_test(t => {
  const frame = document.body.appendChild(document.createElement("iframe"));
  assert_equals(frame.contentDocument.open(), frame.contentDocument);
  assert_equals(frame.contentDocument.documentElement, null);
  frame.contentDocument.write("<div>heya</div>");
  frame.contentDocument.close();
  frame.contentWindow.addEventListener("pageshow", function() {
    t.step(function() {
      assert_true(true, "Got pageshow event");
    });
    t.done();
  });
}, "document.open(), and the pageshow events");
