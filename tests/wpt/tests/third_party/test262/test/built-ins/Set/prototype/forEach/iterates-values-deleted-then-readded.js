// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-set.prototype.foreach
description: >
    Set.prototype.forEach ( callbackfn [ , thisArg ] )

    ...
    7. Repeat for each e that is an element of entries, in original insertion order
      a. If e is not empty, then
        i. Let funcResult be Call(callbackfn, T, «e, e, S»).
        ii. ReturnIfAbrupt(funcResult).
    ...

    NOTE:

    ...
    Values that are deleted after the call to forEach begins and before being visited are not visited unless the value is added again before the forEach call completes.
    ...

---*/


var s = new Set([1, 2, 3]);
var expects = [1, 3, 2];

s.forEach(function(value, entry, set) {
  var expect = expects.shift();

  // Delete `2` before being visited
  if (value === 1) {
    set.delete(2);
  }

  // Re-add `2` before forEach call completes
  if (value === 3) {
    set.add(2);
  }

  assert.sameValue(value, expect);
  assert.sameValue(entry, expect);
  assert.sameValue(set, s);
});

assert.sameValue(expects.length, 0, "The value of `expects.length` is `0`");
