async_test(t => {
  const input = document.body.appendChild(document.createElement("input"));
  let happened = false;
  input.onfocus = t.step_func(e => {
    happened = true;
    assert_equals(e.type, "focus");
    assert_true(e.composed);
  });
  input.focus();
  input.onblur = t.step_func_done(e => {
    assert_true(happened);
    assert_equals(e.type, "blur");
    assert_true(e.composed);
  });
  input.blur();
}, "Focus events are composed");
