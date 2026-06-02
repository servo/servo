test_dynamicOrdered.step(function() {
    assert_execCount(1, 3, "External script element (#2) should have fired third");
});
