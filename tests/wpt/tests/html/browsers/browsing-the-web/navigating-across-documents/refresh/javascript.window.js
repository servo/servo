async_test(t => {
  const frame = document.createElement("iframe");
  t.add_cleanup(() => frame.remove());
  const path = "resources/javascript.asis";
  frame.src = path;
  frame.onload = t.step_func(() => {
    assert_equals(frame.contentWindow.location.href, new URL(path, self.location).href);
    assert_equals(frame.contentWindow.x, 1);
    t.step_timeout(() => {
      assert_equals(frame.contentWindow.x, 1);
      t.done();
    }, 100);
  });
  document.body.appendChild(frame);
}, "Refresh to a javascript: URL should not work");
