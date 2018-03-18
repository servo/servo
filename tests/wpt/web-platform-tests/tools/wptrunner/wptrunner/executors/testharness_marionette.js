window.wrappedJSObject.timeout_multiplier = %(timeout_multiplier)d;
window.wrappedJSObject.explicit_timeout = %(explicit_timeout)d;

window.wrappedJSObject.message_queue = [];

window.wrappedJSObject.setMessageListener = function(func) {
  window.wrappedJSObject.current_listener = func;
  window.wrappedJSObject.addEventListener(
    "message",
    func,
    false
  );
};

window.wrappedJSObject.setMessageListener(function(event) {
  window.wrappedJSObject.message_queue.push(event);
});

window.wrappedJSObject.win = window.wrappedJSObject.open("%(abs_url)s", "%(window_id)s");

window.wrappedJSObject.timer = setTimeout(function() {
  window.wrappedJSObject.win.timeout();
  window.wrappedJSObject.win.close();
}, %(timeout)s);
