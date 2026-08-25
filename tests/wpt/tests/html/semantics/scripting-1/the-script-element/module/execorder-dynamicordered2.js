test_dynamicOrdered.step(function() {
    assert_execCount(1, 2, "External script element (#1) should have fired second");
});
