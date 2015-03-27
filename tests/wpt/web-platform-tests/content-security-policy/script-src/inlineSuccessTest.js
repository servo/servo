var inlineRan = false;

onload = function() {
  test(function() {
        assert_true(inlineRan, 'Unsafe inline script ran.')},
        'Inline script in a script tag should  run with an unsafe-inline directive'
    );
}