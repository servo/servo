async_test(t => {
  const frame = document.createElement("iframe");
  frame.onload = t.step_func(() => {
    const frameW = frame.contentWindow,
          frameD = frame.contentDocument;
    assert_equals(frameW.document, frameD);
    frame.remove();
    assert_equals(frameW.document, frameD);
    t.step_timeout(() => {
      assert_equals(frameW.document, frameD);
      t.done();
    }, 100);
  });
  document.body.append(frame);
}, "Window object's document IDL attribute and discarding the browsing context");
