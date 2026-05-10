// Copyright (C) 2019 Tiancheng "Timothy" Gu. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-math.fround
description: Math.fround should use roundTiesToEven for conversion to binary32.
---*/

// We test five values against Math.fround, with their binary64 representation
// shown:
// a0 := 1.0                = 0x1p+0
// a1 := 1.0000000596046448 = 0x1.000001p+0
// a2 := 1.0000001192092896 = 0x1.000002p+0
// a3 := 1.0000001788139343 = 0x1.000003p+0
// a4 := 1.000000238418579  = 0x1.000004p+0
// a5 := 1.0000002980232239 = 0x1.000005p+0
// a6 := 1.0000003576278687 = 0x1.000006p+0
// (Note: they are separated by 2 ** -24.)
//
// a0, a2, a4, and a6 are all representable exactly in binary32; however, while
// a0 and a4 have even mantissas in binary32, a2 and a6 have an odd mantissa
// when represented in that way.
//
// a1 is exactly halfway between a0 and a2, a3 between a2 and a4, and a5
// between a4 and a6. By roundTiesToEven, Math.fround should favor a0 and a4
// over a2 when they are equally close, and a4 over a6 when they are equally
// close.

var a0 = 1.0;
var a1 = 1.0000000596046448;
var a2 = 1.0000001192092896;
var a3 = 1.0000001788139343;
var a4 = 1.000000238418579;
var a5 = 1.0000002980232239;
var a6 = 1.0000003576278687;

assert.sameValue(Math.fround(a0), a0, 'Math.fround(a0)');
assert.sameValue(Math.fround(a1), a0, 'Math.fround(a1)');
assert.sameValue(Math.fround(a2), a2, 'Math.fround(a2)');
assert.sameValue(Math.fround(a3), a4, 'Math.fround(a3)');
assert.sameValue(Math.fround(a4), a4, 'Math.fround(a4)');
assert.sameValue(Math.fround(a5), a4, 'Math.fround(a5)');
assert.sameValue(Math.fround(a6), a6, 'Math.fround(a6)');

assert.sameValue(Math.fround(-a0), -a0, 'Math.fround(-a0)');
assert.sameValue(Math.fround(-a1), -a0, 'Math.fround(-a1)');
assert.sameValue(Math.fround(-a2), -a2, 'Math.fround(-a2)');
assert.sameValue(Math.fround(-a3), -a4, 'Math.fround(-a3)');
assert.sameValue(Math.fround(-a4), -a4, 'Math.fround(-a4)');
assert.sameValue(Math.fround(-a5), -a4, 'Math.fround(-a5)');
assert.sameValue(Math.fround(-a6), -a6, 'Math.fround(-a6)');
