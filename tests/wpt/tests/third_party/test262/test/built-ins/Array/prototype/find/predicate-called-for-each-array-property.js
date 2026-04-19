// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-array.prototype.find
description: >
  Predicate is called for each array property.
info: |
  22.1.3.8 Array.prototype.find ( predicate[ , thisArg ] )

  ...
  6. If thisArg was supplied, let T be thisArg; else let T be undefined.
  7. Let k be 0.
  8. Repeat, while k < len
    ...
    d. Let testResult be ToBoolean(Call(predicate, T, «kValue, k, O»)).
  ...
---*/

var arr = [undefined, , , 'foo'];
var called = 0;

arr.find(function() {
  called++;
});

assert.sameValue(called, 4);
