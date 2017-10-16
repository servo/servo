function assert_list(list, expectedValues) {
  assert_equals(list.numberOfItems, expectedValues.length);
  for (var index = 0; index < expectedValues.length; ++index)
    assert_equals(list.getItem(index).value, expectedValues[index]);

  assert_throws("IndexSizeError", function() { list.getItem(expectedValues.length); });
}