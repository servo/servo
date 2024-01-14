// This file contains helper methods that are appended to the start of bidder
// and seller worklets.

// Comparison function that checks if two arguments are the same.
// Not intended for use on anything other than built-in types
// (Arrays, objects, and primitive types).
function deepEquals(a, b) {
  if (typeof a !== typeof b)
    return false;
  if (typeof a !== 'object' || a === null || b === null)
    return a === b;

  let aKeys = Object.keys(a);
  if (aKeys.length != Object.keys(b).length)
    return false;
  for (let key of aKeys) {
    if (a.hasOwnProperty(key) != b.hasOwnProperty(key) ||
        !deepEquals(a[key], b[key])) {
      return false;
    }
  }
  return true;
}
