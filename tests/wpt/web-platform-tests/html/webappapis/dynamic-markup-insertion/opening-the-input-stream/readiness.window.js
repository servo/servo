// This tests the behavior of dynamic markup insertion APIs with a document's
// readiness.

async_test(t => {
  const frame = document.body.appendChild(document.createElement("iframe"));
  t.add_cleanup(() => { frame.remove(); });
  frame.src = "/common/blank.html";
  frame.onload = t.step_func_done(() => {
    const states = [];
    frame.contentDocument.onreadystatechange = t.step_func(() => {
      states.push(frame.contentDocument.readyState);
    });
    assert_equals(frame.contentDocument.readyState, "complete");
    assert_array_equals(states, []);

    // When open() is called, it first removes the event listeners and handlers
    // from all nodes in the DOM tree. Then, after a new parser is created and
    // initialized, it changes the current document readiness to "loading".
    // However, because all event listeners are removed, we cannot observe the
    // readystatechange event fired for "loading" inside open().
    frame.contentDocument.open();
    assert_equals(frame.contentDocument.readyState, "loading");
    assert_array_equals(states, []);
  });
}, "document.open() and readiness");
