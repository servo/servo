// META: script=/common/utils.js

async_test((t) => {
  var popup_window = window.open("resources/echo-critical-hint.py?mismatch=true&token="+token());
  assert_not_equals(popup_window, null, "Popup windows not allowed?");
  popup_window.addEventListener('load', (e) => {
    t.step(()=>{assert_equals(popup_window.document.body.textContent, "1")});
    t.done();
  });
}, "Critical-CH navigation")
