/*
 * Any copyright is dedicated to the Public Domain.
 * http://creativecommons.org/licenses/publicdomain/
 */

/*---
description: |
  Number.prototype.toString should use ToInteger on the radix and should throw a RangeError if the radix is bad
info: bugzilla.mozilla.org/show_bug.cgi?id=647385
esid: pending
---*/

function test(r) {
  assert.throws(RangeError, function() {
    5..toString(r);
  });
}

test(Math.pow(2, 32) + 10);
test(55);
