// Copyright 2019 Google, LLC.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: prod-OptionalExpression
description: >
  optional chain in init/test/update of for statement
info: |
  IterationStatement
    for (Expression; Expression; Expression) Statement
features: [optional-chaining]
---*/

// OptionalExpression in test.
let count;
const obj = {a: true};
for (count = 0; obj?.a; count++) {
  if (count > 0) break;
}
assert.sameValue(count, 1);

// OptionalExpression in init/test/update.
let count2 = 0;
const obj2 = undefined;

for (obj?.a; obj2?.a; obj?.a) { count2++; }
assert.sameValue(count2, 0);

for (obj?.a; undefined?.a; obj?.a) { count2++; }
assert.sameValue(count2, 0);

// Short-circuiting
let touched = 0;
const obj3 = {
  get a() {
    count++;
    return undefined; // explicit for clarity
  }
};
for (count = 0; true; obj3?.a?.[touched++]) {
  if (count > 0) { break; }
}
assert.sameValue(count, 1);
assert.sameValue(touched, 0);
