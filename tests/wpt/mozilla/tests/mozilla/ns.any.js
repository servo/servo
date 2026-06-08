// META: title=Namespace bindings

test(function () {
    assert_equals(TestNS.ONE, 1);
    assert_equals(TestNS.TWO, 2);
}, "Namespace constants");

test(function () {
    assert_true(TestNS.testAttribute instanceof TestBinding, "Should be TestBinding");
}, "Namespace attributes");
