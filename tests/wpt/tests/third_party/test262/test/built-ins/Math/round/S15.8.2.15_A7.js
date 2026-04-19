// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    If x is less than or equal to -0 and x is greater than or equal to -0.5,
    Math.round(x) is equal to -0
es5id: 15.8.2.15_A7
description: >
    `Math.round(x)` differs from `Math.floor(x + 0.5)`:

    1) for values in [-0.5; -0]
    2) for 0.5 - Number.EPSILON / 4
    3) for odd integers in [-(2 / Number.EPSILON - 1); -(1 / Number.EPSILON + 1)] or in [1 / Number.EPSILON + 1; 2 / Number.EPSILON - 1]
---*/
assert.sameValue(
  1 / Math.round(-0.5),
  1 / -0,
  'The result of evaluating (1 / Math.round(-0.5)) is expected to be 1 / -0'
);

assert.sameValue(
  1 / Math.round(-0.25),
  1 / -0,
  'The result of evaluating (1 / Math.round(-0.25)) is expected to be 1 / -0'
);

assert.sameValue(1 / Math.round(-0), 1 / -0, 'The result of evaluating (1 / Math.round(-0)) is expected to be 1 / -0');

var x = 0;

// CHECK#4
x = 0.5 - Number.EPSILON / 4;
assert.sameValue(1 / Math.round(x), 1 / 0, 'The result of evaluating (1 / Math.round(x)) is expected to be 1 / 0');

// CHECK#5
x = -(2 / Number.EPSILON - 1);
assert.sameValue(Math.round(x), x, 'Math.round(-(2 / Number.EPSILON - 1)) returns x');

// CHECK#6
x = -(1.5 / Number.EPSILON - 1);
assert.sameValue(Math.round(x), x, 'Math.round(-(1.5 / Number.EPSILON - 1)) returns x');

// CHECK#7
x = -(1 / Number.EPSILON + 1);
assert.sameValue(Math.round(x), x, 'Math.round(-(1 / Number.EPSILON + 1)) returns x');

// CHECK#8
x = 1 / Number.EPSILON + 1;
assert.sameValue(Math.round(x), x, 'Math.round(1 / Number.EPSILON + 1) returns x');

// CHECK#9
x = 1.5 / Number.EPSILON - 1;
assert.sameValue(Math.round(x), x, 'Math.round(1.5 / Number.EPSILON - 1) returns x');

// CHECK#10
x = 2 / Number.EPSILON - 1;
assert.sameValue(Math.round(x), x, 'Math.round(2 / Number.EPSILON - 1) returns x');
