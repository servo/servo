var t1 = async_test("Inline script block");
var t2 = async_test("Inline event handler");

onload = function() {t1.done(); t2.done();};

var t_spv = async_test("Should fire policy violation events");
var block_event_fired = false;
var handler_event_fired = false;
window.addEventListener("securitypolicyviolation", t_spv.step_func(function(e) {
    if (e.violatedDirective == "script-src-elem") {
      assert_false(block_event_fired);
      block_event_fired = true;
    } else if (e.violatedDirective == "script-src-attr") {
      assert_false(handler_event_fired);
      handler_event_fired = true;
    } else {
      assert_unreached("Unexpected directive broken");
    }
    if (block_event_fired && handler_event_fired) {
      t_spv.done();
    }
}));
