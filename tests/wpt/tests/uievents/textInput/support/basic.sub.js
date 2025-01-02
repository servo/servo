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
      let beforeinputEvents = 0;
      let textInputEvents = 0;
      el.addEventListener('beforeinput', t.step_func(e => {
        beforeinputEvents++;
      }));
      el.addEventListener('textInput', t.step_func(e => {
        textInputEvents++;
        assert_equals(beforeinputEvents, 1);
        assert_equals(e.data, expectedData);
        assert_true(e.bubbles);
        assert_true(e.cancelable);
        assert_equals(e.view, window);
        assert_equals(e.detail, 0);
        assert_true(e instanceof window.TextEvent);
      }));
      el.addEventListener('input', t.step_func(e => {
        assert_equals(textInputEvents, 1);
        if (expectedValue === "\n" && !(el instanceof HTMLInputElement) && !(el instanceof HTMLTextAreaElement)) {
          // New paragraph in contenteditable during editing is weird.
          // innerHTML is <div><br></div><div><br></div>
          // ...but later changes to <br>
          // So, check that there's at least one <br>.
          assert_true(getValue(el).indexOf('<br>') > -1);
        } else {
          assert_equals(getValue(el), expectedValue);
        }
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
