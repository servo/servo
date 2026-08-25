// Copyright (C) 2016 The V8 Project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-number.prototype.toprecision
description: >
  Return string values using exponential character
info: |
  Number.prototype.toPrecision ( precision )

  1. Let x be ? thisNumberValue(this value).
  [...]
  5. Let s be the empty String.
  [...]
  9. If x = 0, then
    [...]
  10. Else x ≠ 0,
    [...]
    c. If e < -6 or e ≥ p, then
      [...]
      vii. Return the concatenation of s, m, code unit 0x0065 (LATIN SMALL
      LETTER E), c, and d.
  [...]
---*/

assert.sameValue((10).toPrecision(1), "1e+1");
assert.sameValue((11).toPrecision(1), "1e+1");
assert.sameValue((17).toPrecision(1), "2e+1");
assert.sameValue((19).toPrecision(1), "2e+1");
assert.sameValue((20).toPrecision(1), "2e+1");

assert.sameValue((100).toPrecision(1), "1e+2");
assert.sameValue((1000).toPrecision(1), "1e+3");
assert.sameValue((10000).toPrecision(1), "1e+4");
assert.sameValue((100000).toPrecision(1), "1e+5");

assert.sameValue((100).toPrecision(2), "1.0e+2");
assert.sameValue((1000).toPrecision(2), "1.0e+3");
assert.sameValue((10000).toPrecision(2), "1.0e+4");
assert.sameValue((100000).toPrecision(2), "1.0e+5");

assert.sameValue((1000).toPrecision(3), "1.00e+3");
assert.sameValue((10000).toPrecision(3), "1.00e+4");
assert.sameValue((100000).toPrecision(3), "1.00e+5");

assert.sameValue((42).toPrecision(1), "4e+1");
assert.sameValue((-42).toPrecision(1), "-4e+1");

assert.sameValue((1.2345e+27).toPrecision(1), "1e+27");
assert.sameValue((1.2345e+27).toPrecision(2), "1.2e+27");
assert.sameValue((1.2345e+27).toPrecision(3), "1.23e+27");
assert.sameValue((1.2345e+27).toPrecision(4), "1.234e+27");
assert.sameValue((1.2345e+27).toPrecision(5), "1.2345e+27");
assert.sameValue((1.2345e+27).toPrecision(6), "1.23450e+27");
assert.sameValue((1.2345e+27).toPrecision(7), "1.234500e+27");
assert.sameValue((1.2345e+27).toPrecision(16), "1.234500000000000e+27");
assert.sameValue((1.2345e+27).toPrecision(17), "1.2345000000000000e+27");
assert.sameValue((1.2345e+27).toPrecision(18), "1.23449999999999996e+27");
assert.sameValue((1.2345e+27).toPrecision(19), "1.234499999999999962e+27");
assert.sameValue((1.2345e+27).toPrecision(20), "1.2344999999999999618e+27");
assert.sameValue((1.2345e+27).toPrecision(21), "1.23449999999999996184e+27");

assert.sameValue((-1.2345e+27).toPrecision(1), "-1e+27");
assert.sameValue((-1.2345e+27).toPrecision(2), "-1.2e+27");
assert.sameValue((-1.2345e+27).toPrecision(3), "-1.23e+27");
assert.sameValue((-1.2345e+27).toPrecision(4), "-1.234e+27");
assert.sameValue((-1.2345e+27).toPrecision(5), "-1.2345e+27");
assert.sameValue((-1.2345e+27).toPrecision(6), "-1.23450e+27");
assert.sameValue((-1.2345e+27).toPrecision(7), "-1.234500e+27");
assert.sameValue((-1.2345e+27).toPrecision(16), "-1.234500000000000e+27");
assert.sameValue((-1.2345e+27).toPrecision(17), "-1.2345000000000000e+27");
assert.sameValue((-1.2345e+27).toPrecision(18), "-1.23449999999999996e+27");
assert.sameValue((-1.2345e+27).toPrecision(19), "-1.234499999999999962e+27");
assert.sameValue((-1.2345e+27).toPrecision(20), "-1.2344999999999999618e+27");
assert.sameValue((-1.2345e+27).toPrecision(21), "-1.23449999999999996184e+27");

var n = new Number("1000000000000000000000"); // 1e+21
assert.sameValue((n).toPrecision(1), "1e+21");
assert.sameValue((n).toPrecision(2), "1.0e+21");
assert.sameValue((n).toPrecision(3), "1.00e+21");
assert.sameValue((n).toPrecision(4), "1.000e+21");
assert.sameValue((n).toPrecision(5), "1.0000e+21");
assert.sameValue((n).toPrecision(6), "1.00000e+21");
assert.sameValue((n).toPrecision(7), "1.000000e+21");
assert.sameValue((n).toPrecision(16), "1.000000000000000e+21");
assert.sameValue((n).toPrecision(17), "1.0000000000000000e+21");
assert.sameValue((n).toPrecision(18), "1.00000000000000000e+21");
assert.sameValue((n).toPrecision(19), "1.000000000000000000e+21");
assert.sameValue((n).toPrecision(20), "1.0000000000000000000e+21");
assert.sameValue((n).toPrecision(21), "1.00000000000000000000e+21");

var n = new Number("0.000000000000000000001"); // 1e-21
assert.sameValue((n).toPrecision(1), "1e-21");
assert.sameValue((n).toPrecision(2), "1.0e-21");
assert.sameValue((n).toPrecision(3), "1.00e-21");
assert.sameValue((n).toPrecision(4), "1.000e-21");
assert.sameValue((n).toPrecision(5), "1.0000e-21");
assert.sameValue((n).toPrecision(6), "1.00000e-21");
assert.sameValue((n).toPrecision(7), "1.000000e-21");
assert.sameValue((n).toPrecision(16), "9.999999999999999e-22");
assert.sameValue((n).toPrecision(17), "9.9999999999999991e-22");
assert.sameValue((n).toPrecision(18), "9.99999999999999908e-22");
assert.sameValue((n).toPrecision(19), "9.999999999999999075e-22");
assert.sameValue((n).toPrecision(20), "9.9999999999999990754e-22");
assert.sameValue((n).toPrecision(21), "9.99999999999999907537e-22");

assert.sameValue((0.00000001).toPrecision(1), "1e-8");
assert.sameValue((-0.00000001).toPrecision(1), "-1e-8");
