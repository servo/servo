wildcardPortTestRan = false;

onload = function() {
  test(function() {
        assert_true(wildcardPortTestRan, 'Script should have ran.')},
        "Wildcard port matching works."
    );
}
