// META: global=window,worker

function runTests(data) {
  for (let entry of data) {
    test(function() {
      if (entry.error) {
        assert_throws_js(TypeError, _ => new URLPattern(entry.pattern),
                         'URLPattern() constructor');
        return;
      }

      const pattern = new URLPattern(entry.pattern);

      // First, validate the test() method by converting the expected result to
      // a truthy value.
      assert_equals(pattern.test(entry.input), !!entry.expected,
                    'test() result');

      // Next, start validating the exec() method.
      const result = pattern.exec(entry.input);

      // On a failed match exec() returns null.
      if (!entry.expected) {
        assert_equals(result, entry.expected, 'exec() failed match result');
        return;
      }

      // Next verify the result.input is correct.  This may be a structured
      // URLPatternInit dictionary object or a URL string.
      if (typeof entry.expected.input === 'object') {
        assert_object_equals(result.input, entry.expected.input,
                             'exec() result.input');
      } else {
        assert_equals(result.input, entry.expected.input,
                      'exec() result.input');
      }

      // Next we will compare the URLPatternComponentResult for each of these
      // expected components.
      const component_list = [
        'protocol',
        'username',
        'password',
        'hostname',
        'password',
        'pathname',
        'search',
        'hash',
      ];

      for (let component of component_list) {
        let expected_obj = entry.expected[component];

        // If the test expectations don't include a component object, then
        // we auto-generate one.  This is convenient for the many cases
        // where the pattern has a default wildcard or empty string pattern
        // for a component and the input is essentially empty.
        if (!expected_obj) {
          expected_obj = { input: '', groups: {} };

          // Next, we must treat default wildcards differently than empty string
          // patterns.  The wildcard results in a capture group, but the empty
          // string pattern does not.  The expectation object must list which
          // components should be empty instead of wildcards in
          // |exactly_empty_components|.
          if (!entry.expected.exactly_empty_components ||
              !entry.expected.exactly_empty_components.includes(component)) {
            expected_obj.groups['0'] = '';
          }
        }
        assert_object_equals(result[component], expected_obj,
                             `exec() result for ${component}`);
      }
    }, `Pattern: ${JSON.stringify(entry.pattern)} Input: ${JSON.stringify(entry.input)}`);
  }
}

promise_test(async function() {
  const response = await fetch('resources/urlpatterntestdata.json');
  const data = await response.json();
  runTests(data);
}, 'Loading data...');
