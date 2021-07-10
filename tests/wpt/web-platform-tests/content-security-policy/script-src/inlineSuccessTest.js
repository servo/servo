var t_spv = async_test("Should not fire policy violation events");
window.addEventListener("securitypolicyviolation", t_spv.unreached_func("Should have not fired any securitypolicyviolation event"));

var inlineRan = false;

onload = function() {
  test(function() {
        assert_true(inlineRan, 'Unsafe inline script ran.')},
        'Inline script in a script tag should  run with an unsafe-inline directive'
    );
  t_spv.done();
}