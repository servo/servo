// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-array.prototype.find
description: >
  Predicate called as F.call( thisArg, kValue, k, O ) for each array entry.
info: |
  22.1.3.8 Array.prototype.find ( predicate[ , thisArg ] )

  ...
  6. If thisArg was supplied, let T be thisArg; else let T be undefined.
  7. Let k be 0.
  8. Repeat, while k < len
    ...
    d. Let testResult be ToBoolean(Call(predicate, T, «kValue, k, O»)).
    e. ReturnIfAbrupt(testResult).
  ...
---*/

var arr = ['Mike', 'Rick', 'Leo'];

var results = [];

arr.find(function(kValue, k, O) {
  results.push(arguments);
});

assert.sameValue(results.length, 3);

var result = results[0];
assert.sameValue(result[0], 'Mike');
assert.sameValue(result[1], 0);
assert.sameValue(result[2], arr);
assert.sameValue(result.length, 3);

result = results[1];
assert.sameValue(result[0], 'Rick');
assert.sameValue(result[1], 1);
assert.sameValue(result[2], arr);
assert.sameValue(result.length, 3);

result = results[2];
assert.sameValue(result[0], 'Leo');
assert.sameValue(result[1], 2);
assert.sameValue(result[2], arr);
assert.sameValue(result.length, 3);
