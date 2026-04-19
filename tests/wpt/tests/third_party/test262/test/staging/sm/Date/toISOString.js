/*
 * Any copyright is dedicated to the Public Domain.
 * http://creativecommons.org/licenses/publicdomain/
 */

/*---
description: |
  pending
esid: pending
---*/

function throwsRangeError(t) {
  var date = new Date();
  date.setTime(t);

  assert.throws(RangeError, function() {
    date.toISOString();
  });
}

throwsRangeError(Infinity);
throwsRangeError(-Infinity);
throwsRangeError(NaN);
