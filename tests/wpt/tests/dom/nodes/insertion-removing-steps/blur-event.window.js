test(() => {
  const input = document.body.appendChild(document.createElement('input'));
  input.focus();

  let blurCalled = false;
  input.onblur = e => blurCalled = true;
  input.remove();
  assert_false(blurCalled, "Blur event was not fired");
}, "<input> element does not fire blur event upon DOM removal");

test(() => {
  const button = document.body.appendChild(document.createElement('button'));
  button.focus();

  let blur_called = false;
  let focus_out_called = false;
  let focus_called = false;

  button.onblur = () => { blur_called = true; }
  button.onfocusout = () => { focus_out_called = true; }
  document.body.addEventListener("focus",
    () => { focus_called = true; }, {capture: true});
  button.remove();

  assert_false(blur_called, "Blur event was not fired");
  assert_false(focus_out_called, "FocusOut event was not fired");
  assert_false(focus_called, "Focus was not fired");
}, "<button> element does not fire blur/focusout events upon DOM removal");
