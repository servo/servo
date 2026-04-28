// Copyright (C) 2017 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-dataview.prototype.getbiguint64
description: >
  Return value from Buffer using a clean ArrayBuffer
features: [DataView, ArrayBuffer, BigInt]
---*/

var buffer = new ArrayBuffer(12);
var sample = new DataView(buffer, 0);

assert.sameValue(sample.getBigUint64(0, true), 0n, "sample.getBigUint64(0, true)");
assert.sameValue(sample.getBigUint64(1, true), 0n, "sample.getBigUint64(1, true)");
assert.sameValue(sample.getBigUint64(2, true), 0n, "sample.getBigUint64(2, true)");
assert.sameValue(sample.getBigUint64(3, true), 0n, "sample.getBigUint64(3, true)");
assert.sameValue(sample.getBigUint64(4, true), 0n, "sample.getBigUint64(4, true)");
assert.sameValue(sample.getBigUint64(0, false), 0n, "sample.getBigUint64(0, false)");
assert.sameValue(sample.getBigUint64(1, false), 0n, "sample.getBigUint64(1, false)");
assert.sameValue(sample.getBigUint64(2, false), 0n, "sample.getBigUint64(2, false)");
assert.sameValue(sample.getBigUint64(3, false), 0n, "sample.getBigUint64(3, false)");
assert.sameValue(sample.getBigUint64(4, false), 0n, "sample.getBigUint64(4, false)");
