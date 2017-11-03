var t1 = async_test("Inline script block");
var t2 = async_test("Inline event handler");

onload = function() {t1.done(); t2.done()}

var t_spv = async_test("Should not fire policy violation events");
var test_count = 2;
window.addEventListener("securitypolicyviolation", t_spv.step_func(function(e) {
    assert_equals(e.violatedDirective, "script-src");
    if (--test_count <= 0) {
        t_spv.done();
    }
}));
