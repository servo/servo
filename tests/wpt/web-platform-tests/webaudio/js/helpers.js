function assert_array_approx_equals(actual, expected, epsilon, description)
{
  assert_true(actual.length === expected.length,
              (description + ": lengths differ, expected " + expected.length + " got " + actual.length))

  for (var i=0; i < actual.length; i++) {
    assert_approx_equals(actual[i], expected[i], epsilon, (description + ": element " + i))
  }
}

/*
  Returns an array (typed or not), of the passed array with removed trailing and ending
  zero-valued elements
 */
function trimEmptyElements(array) {
  var start = 0;
  var end = array.length;
  
  while (start < array.length) {
    if (array[start] !== 0) {
      break;
    }
    start++;
  }

  while (end > 0) {
    end--;
    if (array[end] !== 0) {
      break;
    }
  }
  return array.subarray(start, end);
}
