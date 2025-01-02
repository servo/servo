// Usage note: if you use these more than once in a given file, be sure to
// clean up any navigate event listeners, e.g. by using { once: true }, between
// tests.

const TAB_KEY = "\uE004";

export function testFocusWasReset(setupFunc, description) {
  promise_test(async t => {
    setupFunc(t);

    const button = document.body.appendChild(document.createElement("button"));
    const button2 = document.body.appendChild(document.createElement("button"));
    button.tabIndex = 0;
    button2.tabIndex = 0;
    t.add_cleanup(() => {
      button.remove();
      button2.remove();
    });

    assert_equals(document.activeElement, document.body, "Start on body");
    button.focus();
    assert_equals(document.activeElement, button, "focus() worked");

    const { committed, finished } = navigation.navigate("#" + location.hash.substring(1) + "1");

    await committed;
    assert_equals(document.activeElement, button, "Focus stays on the button during the transition");

    await finished.catch(() => {});
    assert_equals(document.activeElement, document.body, "Focus reset after the transition");

    button2.onfocus = t.unreached_func("button2 must not be focused after pressing Tab");
    const focusPromise = waitForFocus(t, button);
    await test_driver.send_keys(document.body, TAB_KEY);
    await focusPromise;
  }, description);
}

export function testFocusWasNotReset(setupFunc, description) {
  promise_test(async t => {
    setupFunc(t);

    const button = document.body.appendChild(document.createElement("button"));
    const button2 = document.body.appendChild(document.createElement("button"));
    button2.tabIndex = 0;
    t.add_cleanup(() => {
      button.remove();
      button2.remove();
    });

    assert_equals(document.activeElement, document.body, "Start on body");
    button.focus();
    assert_equals(document.activeElement, button, "focus() worked");

    const { committed, finished } = navigation.navigate("#" + location.hash.substring(1) + "1");

    await committed;
    assert_equals(document.activeElement, button, "Focus stays on the button during the transition");

    await finished.catch(() => {});
    assert_equals(document.activeElement, button, "Focus stays on the button after the transition");

    button.onfocus = t.unreached_func("button must not be focused after pressing Tab");
    const focusPromise = waitForFocus(t, button2);
    await test_driver.send_keys(document.body, TAB_KEY);
    await focusPromise;
  }, description);
}

function waitForFocus(t, target) {
  return new Promise(resolve => {
    target.addEventListener("focus", () => resolve(), { once: true });
  });
}
