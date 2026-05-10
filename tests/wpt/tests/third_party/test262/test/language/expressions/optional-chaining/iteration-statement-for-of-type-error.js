// Copyright 2019 Google, LLC.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: prod-OptionalExpression
description: >
  optional chain returning undefined in RHS of for of statement
info: |
  IterationStatement
    for (LeftHandSideExpression of Expression) Statement
features: [optional-chaining]
---*/

assert.throws(TypeError, function() {
  for (const key of {}?.a) ;
});

assert.throws(TypeError, function() {
  for (const key of {}?.a) {}
});

const obj = undefined;
assert.throws(TypeError, function() {
  for (const key of obj?.a) {}
});

assert.throws(TypeError, function() {
  for (const key of obj?.a);
});
