var dataScriptRan = false;

var t_spv = async_test("Test that no report violation event was raised");
window.addEventListener("securitypolicyviolation", t_spv.unreached_func("Should not have raised any securitypolicyviolation event"));