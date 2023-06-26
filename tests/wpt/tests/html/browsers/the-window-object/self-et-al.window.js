function delayed_assert_done(t, w, windowProxySelfReference) {
  // Let's make sure nobody is being sneaky
  t.step_timeout(() => {
    t.step_timeout(() => {
      assert_equals(w[windowProxySelfReference], w, `${windowProxySelfReference} got cleared after some time`);
      t.done();
    }, 0);
  }, 0);
}

[
  "frames",
  "globalThis",
  "self",
  "window"
].forEach(windowProxySelfReference => {
  async_test(t => {
    const frame = document.body.appendChild(document.createElement("iframe")),
          otherW = frame.contentWindow;
    assert_equals(otherW[windowProxySelfReference], otherW, `${windowProxySelfReference} is broken`);
    frame.remove();
    assert_equals(otherW[windowProxySelfReference], otherW, `${windowProxySelfReference} got cleared after browsing context removal`);
    assert_true(otherW.closed);

    delayed_assert_done(t, otherW, windowProxySelfReference);
  }, `iframeWindow.${windowProxySelfReference} before and after removal`);

  async_test(t => {
    const otherW = window.open();
    assert_equals(otherW[windowProxySelfReference], otherW, `${windowProxySelfReference} is broken`);
    otherW.onunload = t.step_func(() => {
      assert_equals(otherW[windowProxySelfReference], otherW, `${windowProxySelfReference} got cleared after browsing context unload`);
      t.step_timeout(() => {
        assert_equals(otherW.opener, null); // Ensure browsing context is discarded
        assert_equals(otherW[windowProxySelfReference], otherW, `${windowProxySelfReference} got cleared after browsing context removal`);
        delayed_assert_done(t, otherW, windowProxySelfReference);
      }, 0);
    });
    otherW.close();
    assert_equals(otherW[windowProxySelfReference], otherW, `${windowProxySelfReference} got cleared after browsing context closure`);
    assert_true(otherW.closed);
  }, `popupWindow.${windowProxySelfReference} before, after closing, and after discarding`)
});
