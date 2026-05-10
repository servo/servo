/*
 * Any copyright is dedicated to the Public Domain.
 * http://creativecommons.org/licenses/publicdomain/
 */

/*---
description: |
  Array.prototype.splice, when it deletes elements, should make sure any deleted but not visited elements are suppressed from subsequent enumeration
info: bugzilla.mozilla.org/show_bug.cgi?id=668024
esid: pending
---*/

var arr = [0, 1, 2, 3, 4, 5, , 7];

var seen = [];
var sawOneBeforeThree = true;
for (var p in arr)
{
  if (p === "1")
  {
    // The order of enumeration of properties is unspecified, so technically,
    // it would be kosher to enumerate "1" last, say, such that all properties
    // in the array actually were enumerated, including an index which splice
    // would delete.  Don't flag that case as a failure.  (SpiderMonkey doesn't
    // do this, and neither do any of the other browser engines, but it is
    // permissible behavior.)
    if (seen.indexOf("3") >= 0)
    {
      sawOneBeforeThree = false;
      break;
    }

    arr.splice(2, 3);
  }

  seen.push(p);
}

if (sawOneBeforeThree)
{
  // ES5 12.6.4 states:
  //
  //   If a property that has not yet been visited during enumeration is
  //   deleted, then it will not be visited.
  //
  // So if we haven't seen "3" by the time we see "1", the splice call above
  // will delete "3", and therefore we must not see it.
  assert.sameValue(seen.indexOf("3"), -1);
}
