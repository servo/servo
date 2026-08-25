// Copyright (C) 2017 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-dataview.prototype.setbigint64
description: >
  ToIndex conversions on byteOffset
features: [DataView, ArrayBuffer, BigInt]
---*/

var buffer = new ArrayBuffer(12);
var sample = new DataView(buffer, 0);

var obj1 = {
  valueOf() {
    return 3;
  }
};
var obj2 = {
  toString() {
    return 4;
  }
};

sample.setBigInt64(0, 0n);
sample.setBigInt64(-0, 42n);
assert.sameValue(sample.getBigInt64(0), 42n, "-0");

sample.setBigInt64(3, 0n);
sample.setBigInt64(obj1, 42n);
assert.sameValue(sample.getBigInt64(3), 42n, "object's valueOf");

sample.setBigInt64(4, 0n);
sample.setBigInt64(obj2, 42n);
assert.sameValue(sample.getBigInt64(4), 42n, "object's toString");

sample.setBigInt64(0, 0n);
sample.setBigInt64("", 42n);
assert.sameValue(sample.getBigInt64(0), 42n, "the Empty string");

sample.setBigInt64(0, 0n);
sample.setBigInt64("0", 42n);
assert.sameValue(sample.getBigInt64(0), 42n, "string '0'");

sample.setBigInt64(2, 0n);
sample.setBigInt64("2", 42n);
assert.sameValue(sample.getBigInt64(2), 42n, "string '2'");

sample.setBigInt64(1, 0n);
sample.setBigInt64(true, 42n);
assert.sameValue(sample.getBigInt64(1), 42n, "true");

sample.setBigInt64(0, 0n);
sample.setBigInt64(false, 42n);
assert.sameValue(sample.getBigInt64(0), 42n, "false");

sample.setBigInt64(0, 0n);
sample.setBigInt64(NaN, 42n);
assert.sameValue(sample.getBigInt64(0), 42n, "NaN");

sample.setBigInt64(0, 0n);
sample.setBigInt64(null, 42n);
assert.sameValue(sample.getBigInt64(0), 42n, "null");

sample.setBigInt64(0, 0n);
sample.setBigInt64(0.1, 42n);
assert.sameValue(sample.getBigInt64(0), 42n, "0.1");

sample.setBigInt64(0, 0n);
sample.setBigInt64(0.9, 42n);
assert.sameValue(sample.getBigInt64(0), 42n, "0.9");

sample.setBigInt64(1, 0n);
sample.setBigInt64(1.1, 42n);
assert.sameValue(sample.getBigInt64(1), 42n, "1.1");

sample.setBigInt64(1, 0n);
sample.setBigInt64(1.9, 42n);
assert.sameValue(sample.getBigInt64(1), 42n, "1.9");

sample.setBigInt64(0, 0n);
sample.setBigInt64(-0.1, 42n);
assert.sameValue(sample.getBigInt64(0), 42n, "-0.1");

sample.setBigInt64(0, 0n);
sample.setBigInt64(-0.99999, 42n);
assert.sameValue(sample.getBigInt64(0), 42n, "-0.99999");

sample.setBigInt64(0, 0n);
sample.setBigInt64(undefined, 42n);
assert.sameValue(sample.getBigInt64(0), 42n, "undefined");
