wildcardHostTestRan = false;

onload = function() {
  test(function() {
        assert_false(wildcardHostTestRan, 'Script should not have ran.')},
        "Wildcard host matching works."
    );
}
