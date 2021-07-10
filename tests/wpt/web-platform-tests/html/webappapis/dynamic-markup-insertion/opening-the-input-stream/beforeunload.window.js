// In an earlier version of the HTML Standard, document open steps had "prompt
// to unload document" as a step. Test that this no longer happens.

async_test(t => {
  const frame = document.body.appendChild(document.createElement("iframe"));
  t.add_cleanup(() => frame.remove());
  frame.src = "/common/blank.html";
  frame.onload = t.step_func(() => {
    frame.contentWindow.onbeforeunload = t.unreached_func("beforeunload should not be fired");
    frame.contentDocument.open();
    t.step_timeout(t.step_func_done(() => {
      // If the beforeunload event has still not fired by this point, we
      // consider the test a success. `frame.remove()` above will allow the
      // `load` event to be fired on the top-level Window, thus unblocking
      // testharness.
    }), 500);
  });
}, "document.open() should not fire a beforeunload event");
