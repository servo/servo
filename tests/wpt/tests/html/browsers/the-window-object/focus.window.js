async_test(t => {
  const input = document.body.appendChild(document.createElement("input"));
  input.onfocus = t.step_func(() => {
    const frame = document.body.appendChild(document.createElement("iframe")),
          frameW = frame.contentWindow;
    frameW.onfocus = t.unreached_func();
    frame.remove();
    frameW.focus();
    t.step_timeout(() => {
      assert_equals(document.activeElement, input);
      t.done();
    }, 100);
  });
  input.focus();
});
