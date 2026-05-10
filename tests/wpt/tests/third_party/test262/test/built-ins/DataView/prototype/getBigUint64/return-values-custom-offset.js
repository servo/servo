// Copyright (C) 2017 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-dataview.prototype.getbiguint64
description: >
  Return values from Buffer using a custom offset
features: [DataView, ArrayBuffer, DataView.prototype.setUint8, BigInt]
---*/

var buffer = new ArrayBuffer(16);
var sample = new DataView(buffer, 0);

sample.setUint8(0, 0x27);
sample.setUint8(1, 0x02);
sample.setUint8(2, 0x06);
sample.setUint8(3, 0x02);
sample.setUint8(4, 0x80);
sample.setUint8(5, 0x00);
sample.setUint8(6, 0x80);
sample.setUint8(7, 0x01);
sample.setUint8(8, 0x7f);
sample.setUint8(9, 0x00);
sample.setUint8(10, 0x01);
sample.setUint8(11, 0x02);
sample.setUint8(12, 0x80);
sample.setUint8(13, 0x7f);
sample.setUint8(14, 0xff);
sample.setUint8(15, 0x80);

sample = new DataView(buffer, 4);

assert.sameValue(sample.getBigUint64(0, false), 0x800080017f000102n, "0, false");
assert.sameValue(sample.getBigUint64(1, false), 0x80017f00010280n, "1, false");
assert.sameValue(sample.getBigUint64(2, false), 0x80017f000102807fn, "2, false");
assert.sameValue(sample.getBigUint64(3, false), 0x17f000102807fffn, "3, false");
assert.sameValue(sample.getBigUint64(4, false), 0x7f000102807fff80n, "4, false");

assert.sameValue(sample.getBigUint64(0, true), 0x201007f01800080n, "0, true");
assert.sameValue(sample.getBigUint64(1, true), 0x800201007f018000n, "1, true");
assert.sameValue(sample.getBigUint64(2, true), 0x7f800201007f0180n, "2, true");
assert.sameValue(sample.getBigUint64(3, true), 0xff7f800201007f01n, "3, true");
assert.sameValue(sample.getBigUint64(4, true), 0x80ff7f800201007fn, "4, true");
