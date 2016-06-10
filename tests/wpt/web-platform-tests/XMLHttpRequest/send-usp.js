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

function run_test() {
  var tests = [];
  for (var i = 0; i < 128; i++) {
    // Multiple subtests so that failures can be fine-grained
    tests[i] = async_test("XMLHttpRequest.send(URLSearchParams) ("+i+")");
  }

  // We use a single XHR since this test tends to time out
  // with 128 consecutive fetches when run in parallel
  // with many other WPT tests.
  var x = new XMLHttpRequest();
  x.onload = function() {
    var response_split = x.response.split("&");
    for (var i = 0; i < 128; i++) {
      tests[i].step(function() {
        assert_equals(response_split[i], "a"+i+"="+encode(i));
        tests[i].done();
      });
    }
  }
  x.onerror = function() {
    for (var i = 0; i < 128; i++) {
      (tests[i].unreached_func())();
    }
  }
  x.open("POST", "resources/content.py");
  var usp = new URLSearchParams();
  for (var i = 0; i < 128; i++) {
    usp.append("a"+i, String.fromCharCode(i));
  }
  x.send(usp)
}
