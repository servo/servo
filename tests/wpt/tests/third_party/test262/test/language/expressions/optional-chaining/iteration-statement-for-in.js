// Copyright 2019 Google, LLC.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: prod-OptionalExpression
description: >
  optional chain in test portion of do while statement
info: |
  IterationStatement
    for (LeftHandSideExpression in Expression) Statement
features: [optional-chaining]
---*/
const obj = {
  inner: {
    a: 1,
    b: 2
  }
};
let str = '';
for (const key in obj?.inner) {
  str += key;
}
assert.sameValue('ab', str);
