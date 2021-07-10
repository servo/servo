ECHO_URL = "resources/echo-critical-hint.py"

message_listener = (t, message) =>
  (e) => {
    t.step(()=>{assert_equals(e.data, message)});
    t.done();
  }

make_message_test = (url, message) =>
  (t) => {
    popup_window = window.open("/common/blank.html");
    assert_not_equals(popup_window, null, "Popup windows not allowed?");
    popup_window.addEventListener('message', message_listener(t, message));
    popup_window.location = url;
  }
