importScripts("{{location[server]}}/resources/testharness.js");

test(t => {
  importScripts("https://{{hosts[][www]}}:{{ports[https][1]}}" +
                "/content-security-policy/support/testharness-helper.js");
}, "Cross-origin `importScripts()` not blocked in " + self.location.protocol +
     " withour CSP");

test(t => {
  assert_equals(2, eval("1+1"));
  assert_equals(2, (new Function("return 1+1;"))());
}, "`eval()` not blocked in " + self.location.protocol +
    " without CSP");

async_test(t => {
  self.callback = t.step_func_done();

  setTimeout("self.callback();", 1);
  setTimeout(t.step_func(_ =>
      assert_unreached("callback not called.")), 2);
}, "`setTimeout([string])` not blocked in " + self.location.protocol +
           " without CSP");

done();
