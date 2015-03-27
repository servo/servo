var t1 = async_test("Inline script block");
var t2 = async_test("Inline event handler");

onload = function() {t1.done(); t2.done()}