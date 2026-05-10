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
---*/

var s = new Set([1]);
var counter = 0;

assert.throws(Error, function() {
  s.forEach(function() {
    counter++;
    throw new Error();
  });
});

assert.sameValue(counter, 1, "`forEach` is not a no-op");
