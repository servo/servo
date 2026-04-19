// Copyright (C) 2021 Microsoft. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-array.prototype.findlastindex
description: >
  Predicate called as F.call( thisArg, kValue, k, O ) for each array entry.
info: |
  Array.prototype.findLastIndex ( predicate[ , thisArg ] )

  ...
  4. Let k be len - 1.
  5. Repeat, while k ≥ 0,
    ...
    c. Let testResult be ! ToBoolean(? Call(predicate, thisArg, « kValue, 𝔽(k), O »)).
  ...
features: [array-find-from-last]
---*/

var arr = ['Mike', 'Rick', 'Leo'];

var results = [];

arr.findLastIndex(function() {
  results.push(arguments);
});

assert.sameValue(results.length, 3);

var result = results[0];
assert.sameValue(result[0], 'Leo');
assert.sameValue(result[1], 2);
assert.sameValue(result[2], arr);
assert.sameValue(result.length, 3);

result = results[1];
assert.sameValue(result[0], 'Rick');
assert.sameValue(result[1], 1);
assert.sameValue(result[2], arr);
assert.sameValue(result.length, 3);

result = results[2];
assert.sameValue(result[0], 'Mike');
assert.sameValue(result[1], 0);
assert.sameValue(result[2], arr);
assert.sameValue(result.length, 3);
