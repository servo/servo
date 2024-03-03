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

  let blurCalled = false;
  button.onblur = e => blurCalled = true;
  button.remove();
  assert_false(blurCalled, "Blur event was not fired");
}, "<button> element does not fire blur event upon DOM removal");
