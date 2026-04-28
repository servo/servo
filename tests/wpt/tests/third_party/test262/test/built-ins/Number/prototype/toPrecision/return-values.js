// Copyright (C) 2016 The V8 Project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-number.prototype.toprecision
description: >
  Return regular string values
info: |
  Number.prototype.toPrecision ( precision )

  1. Let x be ? thisNumberValue(this value).
  [...]
  5. Let s be the empty String.
  [...]
  11. If e = p-1, return the concatenation of the Strings s and m.
  12. If e ≥ 0, then
    a. Let m be the concatenation of the first e+1 elements of m, the code unit
    0x002E (FULL STOP), and the remaining p- (e+1) elements of m.
  13. Else e < 0,
    a. Let m be the String formed by the concatenation of code unit 0x0030
    (DIGIT ZERO), code unit 0x002E (FULL STOP), -(e+1) occurrences of code unit
    0x0030 (DIGIT ZERO), and the String m.
  14. Return the String that is the concatenation of s and m. 
---*/

assert.sameValue((7).toPrecision(1), "7");
assert.sameValue((7).toPrecision(2), "7.0");
assert.sameValue((7).toPrecision(3), "7.00");
assert.sameValue((7).toPrecision(19), "7.000000000000000000");
assert.sameValue((7).toPrecision(20), "7.0000000000000000000");
assert.sameValue((7).toPrecision(21), "7.00000000000000000000");

assert.sameValue((-7).toPrecision(1), "-7");
assert.sameValue((-7).toPrecision(2), "-7.0");
assert.sameValue((-7).toPrecision(3), "-7.00");
assert.sameValue((-7).toPrecision(19), "-7.000000000000000000");
assert.sameValue((-7).toPrecision(20), "-7.0000000000000000000");
assert.sameValue((-7).toPrecision(21), "-7.00000000000000000000");

assert.sameValue((10).toPrecision(2), "10");
assert.sameValue((11).toPrecision(2), "11");
assert.sameValue((17).toPrecision(2), "17");
assert.sameValue((19).toPrecision(2), "19");
assert.sameValue((20).toPrecision(2), "20");

assert.sameValue((-10).toPrecision(2), "-10");
assert.sameValue((-11).toPrecision(2), "-11");
assert.sameValue((-17).toPrecision(2), "-17");
assert.sameValue((-19).toPrecision(2), "-19");
assert.sameValue((-20).toPrecision(2), "-20");

assert.sameValue((42).toPrecision(2), "42");
assert.sameValue((-42).toPrecision(2), "-42");

assert.sameValue((100).toPrecision(3), "100");
assert.sameValue((100).toPrecision(7), "100.0000");
assert.sameValue((1000).toPrecision(7), "1000.000");
assert.sameValue((10000).toPrecision(7), "10000.00");
assert.sameValue((100000).toPrecision(7), "100000.0");

assert.sameValue((0.000001).toPrecision(1), "0.000001");
assert.sameValue((0.000001).toPrecision(2), "0.0000010");
assert.sameValue((0.000001).toPrecision(3), "0.00000100");
