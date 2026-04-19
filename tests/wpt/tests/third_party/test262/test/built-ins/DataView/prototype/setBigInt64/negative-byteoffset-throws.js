// Copyright (C) 2017 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-dataview.prototype.setbigint64
description: >
  Throws a RangeError if getIndex < 0
features: [DataView, ArrayBuffer, BigInt]
---*/

var buffer = new ArrayBuffer(12);
var sample = new DataView(buffer, 0);

assert.throws(RangeError, function() {
  sample.setBigInt64(-1, 39n);
}, "DataView access at index -1 should throw");
assert.sameValue(sample.getBigInt64(0), 0n, "-1 - no value was set");

assert.throws(RangeError, function() {
  sample.setBigInt64(-Infinity, 39n);
}, "DataView access at index -Infinity should throw");
assert.sameValue(sample.getBigInt64(0), 0n, "-Infinity - no value was set");
