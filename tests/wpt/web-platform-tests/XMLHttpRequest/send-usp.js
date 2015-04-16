function encode(n) {
  if (n === 0x20) {
    return "\x2B";
  }

  if (n === 0x2A || n === 0x2D || n === 0x2E ||
      (0x30 <= n && n <= 0x39) || (0x41 <= n && n <= 0x5A) ||
      n === 0x5F || (0x61 <= n && n <= 0x7A)) {
    return String.fromCharCode(n);
  }

  var s = n.toString(16).toUpperCase();
  return "%" + (s.length === 2 ? s : '0' + s);
}

function do_test(n) {
  async_test(function() {
    var x = new XMLHttpRequest();
    x.onload = this.step_func_done(function(e) {
      assert_equals(x.response, "a=" + encode(n))
    });
    x.onerror = this.unreached_func();
    x.open("POST", "resources/content.py");
    var usp = new URLSearchParams();
    usp.append("a", String.fromCharCode(n));
    x.send(usp)
  }, "XMLHttpRequest.send(URLSearchParams) (" + n + ")");
}

function run_test() {
  var i = 0;
  add_result_callback(function() {
    if (++i === 128) {
      return;
    }
    do_test(i);
  });
  do_test(i);
}
