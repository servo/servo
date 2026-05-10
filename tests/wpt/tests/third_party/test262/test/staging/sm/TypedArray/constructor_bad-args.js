// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: |
  pending
esid: pending
---*/
// Bug 1227207

var AB = new ArrayBuffer(12);	// Length divides 4
var BC = new ArrayBuffer(14);	// Length does not divide 4

assert.throws(RangeError, () => new Int32Array(AB, -1)); // 22.2.4.5 #8
assert.throws(RangeError, () => new Int32Array(AB, 2));  // 22.2.4.5 #10
assert.throws(RangeError, () => new Int32Array(BC));	  // 22.2.4.5 #13.a
assert.throws(RangeError, () => new Int32Array(AB, 16)); // 22.2.4.5 #13.c
assert.throws(RangeError, () => new Int32Array(AB, 0, 4)); // 22.2.4.5 #14.c

