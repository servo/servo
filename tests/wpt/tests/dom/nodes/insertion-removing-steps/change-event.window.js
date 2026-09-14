// META: spec=https://html.spec.whatwg.org/multipage/infrastructure.html#dom-trees:concept-node-remove-ext
// META: script=/resources/testdriver.js
// META: script=/resources/testdriver-vendor.js
// META: script=/resources/testdriver-actions.js

promise_test(async () => {
  const input = document.body.appendChild(document.createElement('input'));
  let changeCalled = false;
  input.onchange = () => {
    changeCalled = true;
  };
  input.focus();
  await test_driver.send_keys(input, 'a');
  input.remove();
  assert_false(changeCalled, 'Change event was not fired');
}, '<input> element does not fire change event upon DOM removal');

promise_test(async () => {
  const textarea =
      document.body.appendChild(document.createElement('textarea'));
  let changeCalled = false;
  textarea.onchange = () => {
    changeCalled = true;
  };
  textarea.focus();
  await test_driver.send_keys(textarea, 'a');
  textarea.remove();
  assert_false(changeCalled, 'Change event was not fired');
}, '<textarea> element does not fire change event upon DOM removal');
