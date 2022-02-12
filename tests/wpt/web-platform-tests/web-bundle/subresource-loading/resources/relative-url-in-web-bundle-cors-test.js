promise_test(async (t) => {
  assert_array_equals(
    loaded_scripts,
    [
      'relative-url-file.js',
      'start-with-double-slash-cors.js',
      'start-with-slash.js',
      'subdirectory-path.js',
      'starts-with-two-dots.js',
    ]);
  assert_array_equals(
    failed_scripts,
    [
      'starts-with-two-dots-out-of-scope.js',
    ]);
},
'Relative Url in web bundle.');
