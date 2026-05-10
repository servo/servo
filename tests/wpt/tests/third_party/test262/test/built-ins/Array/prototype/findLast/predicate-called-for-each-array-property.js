// Copyright (C) 2021 Microsoft. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-array.prototype.findlast
description: >
  Predicate is called for each array property.
info: |
  Array.prototype.findLast ( predicate[ , thisArg ] )

  ...
  4. Let k be len - 1.
  5. Repeat, while k ≥ 0,
    ...
    c. Let testResult be ! ToBoolean(? Call(predicate, thisArg, « kValue, 𝔽(k), O »)).
  ...
features: [array-find-from-last]
---*/

var arr = [undefined, , , 'foo'];
var called = 0;

arr.findLast(function() {
  called++;
});

assert.sameValue(called, 4);
