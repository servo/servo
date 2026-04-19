// Copyright 2019 Google, LLC.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: prod-OptionalExpression
description: >
  optional chain in test portion of while statement
info: |
  IterationStatement
    while (Expression) Statement
features: [optional-chaining]
---*/
let count = 0;
const obj = {a: true};
while (obj?.a) {
  count++;
  break;
}
assert.sameValue(1, count);
