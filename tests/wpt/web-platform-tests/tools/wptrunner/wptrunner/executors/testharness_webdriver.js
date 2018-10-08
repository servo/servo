window.timeout_multiplier = %(timeout_multiplier)d;
window.url = "%(url)s";
window.win = window.open("%(abs_url)s", "%(window_id)s");

window.message_queue = [];
window.testdriver_callback = null;

if (%(timeout)s != null) {
  window.timer = setTimeout(function() {
    window.win.timeout();
    window.win.close();
  }, %(timeout)s);
}
