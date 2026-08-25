/*
 * Any copyright is dedicated to the Public Domain.
 * http://creativecommons.org/licenses/publicdomain/
 */

/*---
description: |
  Array.prototype.splice should define, not set, the elements of the array it returns
info: bugzilla.mozilla.org/show_bug.cgi?id=668024
esid: pending
---*/

Object.defineProperty(Object.prototype, 2,
  {
    set: function(v)
    {
      throw new Error("setter on Object.prototype called!");
    },
    get: function() { return "fnord"; },
    enumerable: false,
    configurable: true
  });

var arr = [0, 1, 2, 3, 4, 5];
var removed = arr.splice(0, 6);

assert.sameValue(arr.length, 0);
assert.sameValue(removed.length, 6);
assert.sameValue(removed[0], 0);
assert.sameValue(removed[1], 1);
assert.sameValue(removed[2], 2);
assert.sameValue(removed[3], 3);
assert.sameValue(removed[4], 4);
assert.sameValue(removed[5], 5);
