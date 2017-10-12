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
