// META: title=Namespace bindings

test(function () {
    assert_equals(TestNS.ONE, 1);
    assert_equals(TestNS.TWO, 2);
}, "Namespace constants");
