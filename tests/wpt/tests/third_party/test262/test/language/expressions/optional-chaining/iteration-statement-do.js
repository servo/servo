// Copyright 2019 Google, LLC.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: prod-OptionalExpression
description: >
  optional chain in test portion of do while statement
info: |
  IterationStatement
    do Statement while (OptionalExpression)
features: [optional-chaining]
---*/
let count = 0;
const obj = {a: true};
do {
  count++;
  break;
} while (obj?.a);
assert.sameValue(1, count);
