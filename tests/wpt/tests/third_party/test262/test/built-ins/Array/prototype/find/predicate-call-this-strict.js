// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-array.prototype.find
description: >
  Predicate thisArg as F.call( thisArg, kValue, k, O ) for each array entry.
info: |
  22.1.3.8 Array.prototype.find ( predicate[ , thisArg ] )

  ...
  8. Repeat, while k < len
    ...
    d. Let testResult be ToBoolean(Call(predicate, T, «kValue, k, O»)).
    e. ReturnIfAbrupt(testResult).
  ...
flags: [onlyStrict]
---*/

var result;

[1].find(function(kValue, k, O) {
  result = this;
});

assert.sameValue(result, undefined);

var o = {};
[1].find(function() {
  result = this;
}, o);

assert.sameValue(result, o);
