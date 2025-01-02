// This tests whether the insertion point gets reset before or after the readystatechange event.
// See https://github.com/whatwg/html/pull/6613#discussion_r620171070.
// Recall that resetting the insertion point means that document.write() performs the document open
// steps and blows away previous content in the document.

async_test(t => {
  const frame = document.body.appendChild(document.createElement("iframe"));
  t.add_cleanup(() => { frame.remove(); });
  frame.src = "../opening-the-input-stream/resources/dummy.html";
  frame.onload = t.step_func_done(() => {
    const states = [];
    frame.contentDocument.onreadystatechange = t.step_func(() => {
      if (frame.contentDocument.readyState === "interactive") {
        assert_not_equals(frame.contentDocument.textContent, "", "Precondition check: dummy document is not empty");

        frame.contentDocument.write("Some text");

        // If the insertion point is reset before the readystatechange handler, then the
        // document.write() call above will blow away the text originally in dummy.html, leaving only what we wrote.
        assert_equals(frame.contentDocument.textContent, "Some text");
      }
    });
  });
}, "document.write() during readystatechange to interactive");
