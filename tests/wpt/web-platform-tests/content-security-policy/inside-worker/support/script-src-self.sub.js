importScripts("{{location[server]}}/resources/testharness.js");
importScripts("{{location[server]}}/content-security-policy/support/testharness-helper.js");

test(t => {
  self.a = false;
  assert_throws("NetworkError",
                _ => importScripts("http://{{domains[www]}}:{{ports[http][1]}}/content-security-policy/support/var-a.js"),
                "importScripts should throw `NetworkError`");
  assert_false(self.a);
}, "Cross-origin `importScripts()` blocked in " + self.location.protocol + self.location.search);

test(t => {
  assert_throws(EvalError(),
                _ => eval("1 + 1"),
                "`eval()` should throw 'EvalError'.");

  assert_throws(EvalError(),
                _ => new Function("1 + 1"),
                "`new Function()` should throw 'EvalError'.");
}, "`eval()` blocked in " + self.location.protocol + self.location.search);

async_test(t => {
  waitUntilCSPEventForEval(t, 27)
    .then(t.step_func_done());

  self.setTimeoutTest = t;
  var result = setTimeout("(self.setTimeoutTest.unreached_func('setTimeout([string]) should not execute.'))()", 1);
  assert_equals(result, 0);
}, "`setTimeout([string])` blocked in " + self.location.protocol + self.location.search);

done();
