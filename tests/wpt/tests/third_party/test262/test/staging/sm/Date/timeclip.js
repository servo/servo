/*
 * Any copyright is dedicated to the Public Domain.
 * http://creativecommons.org/licenses/publicdomain/
 */

/*---
description: |
  TimeClip behavior for very large numbers
info: bugzilla.mozilla.org/show_bug.cgi?id=747197
esid: pending
---*/

function addToLimit(n) { return 8.64e15 + n; }

assert.sameValue(8.64e15 === addToLimit(0.0), true);
assert.sameValue(8.64e15 === addToLimit(0.5), true);
assert.sameValue(8.64e15 === addToLimit(0.5000000000000001), false);

var times =
  [Number.MAX_VALUE,
   -Number.MAX_VALUE,
   Infinity,
   -Infinity,
   addToLimit(0.5000000000000001),
   -addToLimit(0.5000000000000001)];

for (var i = 0, len = times.length; i < len; i++)
{
  var d = new Date();
  assert.sameValue(d.setTime(times[i]), NaN, "times[" + i + "]");
  assert.sameValue(d.getTime(), NaN);
  assert.sameValue(d.valueOf(), NaN);
}
