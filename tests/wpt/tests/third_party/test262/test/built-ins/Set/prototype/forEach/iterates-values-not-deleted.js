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

    callbackfn should be a function that accepts three arguments. forEach calls callbackfn once for each value present in the set object, in value insertion order. callbackfn is called only for values of the Set which actually exist; it is not called for keys that have been deleted from the set.

---*/

var expects = [1, 3];
var s = new Set([1, 2, 3]);

s.delete(2);

s.forEach(function(value, entry, set) {
  var expect = expects.shift();

  assert.sameValue(value, expect);
  assert.sameValue(entry, expect);
  assert.sameValue(set, s);
});

assert.sameValue(expects.length, 0, "`forEach` is not a no-op");
