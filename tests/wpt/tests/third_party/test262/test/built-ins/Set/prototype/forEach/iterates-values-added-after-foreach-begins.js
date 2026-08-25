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
    New values added after the call to forEach begins are visited.

---*/


var s = new Set([1]);
var expects = [1, 2, 3];

s.forEach(function(value, entry, set) {
  var expect = expects.shift();

  if (value === 1) {
    set.add(2);
  }

  if (value === 2) {
    set.add(3);
  }

  assert.sameValue(value, expect);
  assert.sameValue(entry, expect);
  assert.sameValue(set, s);
});

assert.sameValue(expects.length, 0, "The value of `expects.length` is `0`");
