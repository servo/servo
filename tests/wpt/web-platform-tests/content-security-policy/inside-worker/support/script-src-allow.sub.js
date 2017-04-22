importScripts("{{location[server]}}/resources/testharness.js");

test(t => {
  importScripts("http://{{domains[www]}}:{{ports[http][1]}}/content-security-policy/support/testharness-helper.js");
}, "Cross-origin `importScripts()` not blocked in " + self.location.protocol + self.location.search);

test(t => {
  assert_equals(2, eval("1+1"));
  assert_equals(2, (new Function("return 1+1;"))());
}, "`eval()` not blocked in " + self.location.protocol + self.location.search);

async_test(t => {
  self.callback = t.step_func_done();

  setTimeout("self.callback();", 1);
}, "`setTimeout([string])` not blocked in " + self.location.protocol + self.location.search);

done();
