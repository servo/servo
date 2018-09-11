function assert_equal_to_array(table, expected, message) {
  assert_equals(table.length, expected.length, `${message}: length`);
  assert_throws(new RangeError(), () => table.get(-1), `${message}: table.get(-1)`);
  for (let i = 0; i < expected.length; ++i) {
    assert_equals(table.get(i), expected[i], `${message}: table.get(${i} of ${expected.length})`);
  }
  assert_throws(new RangeError(), () => table.get(expected.length),
                `${message}: table.get(${expected.length} of ${expected.length})`);
  assert_throws(new RangeError(), () => table.get(expected.length + 1),
                `${message}: table.get(${expected.length + 1} of ${expected.length})`);
}
