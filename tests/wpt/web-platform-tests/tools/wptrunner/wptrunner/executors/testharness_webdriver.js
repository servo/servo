window.timeout_multiplier = %(timeout_multiplier)d;

window.message_queue = [];

window.setMessageListener = function(func) {
  window.current_listener = func;
  window.addEventListener(
    "message",
    func,
    false
  );
};

window.setMessageListener(function(event) {
  window.message_queue.push(event);
});

window.win = window.open("%(abs_url)s", "%(window_id)s");

if (%(timeout)s != null) {
  window.timer = setTimeout(function() {
    window.win.timeout();
    window.win.close();
  }, %(timeout)s);
}
