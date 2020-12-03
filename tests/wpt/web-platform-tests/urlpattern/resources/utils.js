function runTest(pattern, expected_list) {
  const p = new URLPattern(pattern);
  for (let entry of expected_list) {
    assert_equals(p.test(entry.input), entry.expected,
                  `input: ${JSON.stringify(entry.input)}`);
  }
}
