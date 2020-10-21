// META: script=/common/utils.js

async_test((t) => {
  var popup_window = window.open("resources/echo-critical-hint.py?token="+token());
  assert_not_equals(popup_window, null, "Popup windows not allowed?");
  popup_window.addEventListener('load', (e) => {
    t.step(()=>{assert_equals(popup_window.document.body.textContent, "2")});
    t.done();
  });
}, "Critical-CH navigation")
