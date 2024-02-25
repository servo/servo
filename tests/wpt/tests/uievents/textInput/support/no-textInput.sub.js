const els = document.querySelectorAll('.test-el');
const key = "{{GET[key]}}";
const keyRaw = keyMapping[key] || key;
const expectedData = key === "Enter" ? "\n" : key;
const selectionStart = {{GET[selectionStart]}};
const selectionEnd = {{GET[selectionEnd]}};
const expectedValue = "{{GET[expectedValue]}}";

for (const el of els) {
  promise_test(t => {
    return new Promise((resolve, reject) => {
      el.addEventListener('textInput', reject);
      el.addEventListener('keyup', t.step_func(e => {
        if (e.key !== key) {
          return;
        }
        assert_equals(getValue(el), expectedValue);
        resolve();
      }));
      el.onfocus = t.step_func(e => {
        if (window.test_driver) {
          test_driver.send_keys(el, keyRaw);
        }
      });
      el.focus();
      setSelection(el, selectionStart, selectionEnd);
    });
  }, `${document.title}, ${elDesc(el)}`);
}
