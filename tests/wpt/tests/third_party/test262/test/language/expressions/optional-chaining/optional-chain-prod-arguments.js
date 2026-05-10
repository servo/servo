// Copyright 2020 Salesforce.com, Inc. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: prod-OptionalExpression
description: >
  Productions for ?. Arguments
info: |
  OptionalChain[Yield, Await]:
    ?. Arguments
features: [optional-chaining]
---*/

function fn(arg1, arg2, arg3 = 0) {
  return arg1 + arg2 + arg3;
}

assert.sameValue(fn?.(10, 20), 30, 'regular');
assert.sameValue(String?.(42), '42', 'built-in');
assert.sameValue(fn ?. (...[10, 20, 40]), 70, 'spread');
