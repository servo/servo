wildcardHostTestRan = false;

onload = function() {
  test(function() {
        assert_true(wildcardHostTestRan, 'Script should have ran.')},
        "Wildcard host matching works."
    );
}
