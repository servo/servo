// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-date.prototype.valueof
info: |
    Result of ToInteger(value) conversion is the result of computing
    sign(ToNumber(value)) * floor(abs(ToNumber(value)))
es5id: 9.4_A3_T1
description: For testing constructor Date(Number) is used
---*/

// CHECK#1
var d1 = new Date(6.54321);
assert.sameValue(d1.valueOf(), 6, 'd1.valueOf() must return 6');

// CHECK#2
var d2 = new Date(-6.54321);
assert.sameValue(d2.valueOf(), -6, 'd2.valueOf() must return -6');

// CHECK#3
var d3 = new Date(6.54321e2);
assert.sameValue(d3.valueOf(), 654, 'd3.valueOf() must return 654');

// CHECK#4
var d4 = new Date(-6.54321e2);
assert.sameValue(d4.valueOf(), -654, 'd4.valueOf() must return -654');

// CHECK#5
var d5 = new Date(0.654321e1);
assert.sameValue(d5.valueOf(), 6, 'd5.valueOf() must return 6');

// CHECK#6
var d6 = new Date(-0.654321e1);
assert.sameValue(d6.valueOf(), -6, 'd6.valueOf() must return -6');

// CHECK#7
var d7 = new Date(true);
assert.sameValue(d7.valueOf(), 1, 'd7.valueOf() must return 1');

// CHECK#8
var d8 = new Date(false);
assert.sameValue(d8.valueOf(), 0, 'd8.valueOf() must return 0');

// CHECK#9
var d9 = new Date(1.23e15);
assert.sameValue(d9.valueOf(), 1.23e15, 'd9.valueOf() must return 1.23e15');

// CHECK#10
var d10 = new Date(-1.23e15);
assert.sameValue(d10.valueOf(), -1.23e15, 'd10.valueOf() must return -1.23e15');

// CHECK#11
var d11 = new Date(1.23e-15);
assert.sameValue(d11.valueOf(), 0, 'd11.valueOf() must return 0');

// CHECK#12
var d12 = new Date(-1.23e-15);
assert.sameValue(d12.valueOf(), 0, 'd12.valueOf() must return 0');
