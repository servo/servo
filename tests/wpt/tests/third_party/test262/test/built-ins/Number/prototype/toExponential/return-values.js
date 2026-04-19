// Copyright (C) 2016 The V8 Project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-number.prototype.toexponential
description: >
  Return regular string values
---*/

assert.sameValue((123.456).toExponential(0), "1e+2");
assert.sameValue((123.456).toExponential(1), "1.2e+2");
assert.sameValue((123.456).toExponential(2), "1.23e+2");
assert.sameValue((123.456).toExponential(3), "1.235e+2");
assert.sameValue((123.456).toExponential(4), "1.2346e+2");
assert.sameValue((123.456).toExponential(5), "1.23456e+2");
assert.sameValue((123.456).toExponential(6), "1.234560e+2");
assert.sameValue((123.456).toExponential(7), "1.2345600e+2");
assert.sameValue((123.456).toExponential(17), "1.23456000000000003e+2");
assert.sameValue((123.456).toExponential(20), "1.23456000000000003070e+2");

assert.sameValue((-123.456).toExponential(0), "-1e+2");
assert.sameValue((-123.456).toExponential(1), "-1.2e+2");
assert.sameValue((-123.456).toExponential(2), "-1.23e+2");
assert.sameValue((-123.456).toExponential(3), "-1.235e+2");
assert.sameValue((-123.456).toExponential(4), "-1.2346e+2");
assert.sameValue((-123.456).toExponential(5), "-1.23456e+2");
assert.sameValue((-123.456).toExponential(6), "-1.234560e+2");
assert.sameValue((-123.456).toExponential(7), "-1.2345600e+2");
assert.sameValue((-123.456).toExponential(17), "-1.23456000000000003e+2");
assert.sameValue((-123.456).toExponential(20), "-1.23456000000000003070e+2");

assert.sameValue((0.0001).toExponential(0), "1e-4");
assert.sameValue((0.0001).toExponential(1), "1.0e-4");
assert.sameValue((0.0001).toExponential(2), "1.00e-4");
assert.sameValue((0.0001).toExponential(3), "1.000e-4");
assert.sameValue((0.0001).toExponential(4), "1.0000e-4");
assert.sameValue((0.0001).toExponential(16), "1.0000000000000000e-4");
assert.sameValue((0.0001).toExponential(17), "1.00000000000000005e-4");
assert.sameValue((0.0001).toExponential(18), "1.000000000000000048e-4");
assert.sameValue((0.0001).toExponential(19), "1.0000000000000000479e-4");
assert.sameValue((0.0001).toExponential(20), "1.00000000000000004792e-4");

assert.sameValue((0.9999).toExponential(0), "1e+0");
assert.sameValue((0.9999).toExponential(1), "1.0e+0");
assert.sameValue((0.9999).toExponential(2), "1.00e+0");
assert.sameValue((0.9999).toExponential(3), "9.999e-1");
assert.sameValue((0.9999).toExponential(4), "9.9990e-1");
assert.sameValue((0.9999).toExponential(16), "9.9990000000000001e-1");
assert.sameValue((0.9999).toExponential(17), "9.99900000000000011e-1");
assert.sameValue((0.9999).toExponential(18), "9.999000000000000110e-1");
assert.sameValue((0.9999).toExponential(19), "9.9990000000000001101e-1");
assert.sameValue((0.9999).toExponential(20), "9.99900000000000011013e-1");

assert.sameValue((25).toExponential(0), "3e+1");
assert.sameValue((12345).toExponential(3), "1.235e+4");
