var callback = arguments[arguments.length - 1];
var loaded = false;

window.timeout_multiplier = %(timeout_multiplier)d;
window.url = "%(url)s";
window.win = window.open("%(abs_url)s", "%(window_id)s");
window.win.addEventListener('DOMContentLoaded', (e) => {
  callback();
});


window.message_queue = [];
window.testdriver_callback = null;

if (%(timeout)s != null) {
  window.timer = setTimeout(function() {
    window.win.timeout();
  }, %(timeout)s);
}
