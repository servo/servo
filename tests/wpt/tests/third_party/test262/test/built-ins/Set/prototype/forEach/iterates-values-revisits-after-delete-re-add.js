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
    a value will be revisited if it is deleted after it has been visited and then re-added before the forEach call completes.
    ...

---*/


var s = new Set([1, 2, 3]);
var expects = [1, 2, 3, 1];

s.forEach(function(value, entry, set) {
  var expect = expects.shift();

  // Delete `1` after visit
  if (value === 2) {
    set.delete(1);
  }

  // Re-add `1`
  if (value === 3) {
    set.add(1);
  }

  assert.sameValue(value, expect);
  assert.sameValue(entry, expect);
  assert.sameValue(set, s);
});

assert.sameValue(expects.length, 0, "The value of `expects.length` is `0`");
