// Copyright (C) 2017 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-dataview.prototype.setbigint64
description: >
  Boolean littleEndian argument coerced in ToBoolean
features: [DataView, ArrayBuffer, Symbol, BigInt]
---*/

var buffer = new ArrayBuffer(8);
var sample = new DataView(buffer, 0);

// False
sample.setBigInt64(0, 1n);
assert.sameValue(sample.getBigInt64(0), 1n, "no arg");
sample.setBigInt64(0, 2n, undefined);
assert.sameValue(sample.getBigInt64(0), 2n, "undefined");
sample.setBigInt64(0, 3n, null);
assert.sameValue(sample.getBigInt64(0), 3n, "null");
sample.setBigInt64(0, 4n, 0);
assert.sameValue(sample.getBigInt64(0), 4n, "0");
sample.setBigInt64(0, 5n, "");
assert.sameValue(sample.getBigInt64(0), 5n, "the empty string");

// True
sample.setBigInt64(0, 6n, {});
assert.sameValue(sample.getBigInt64(0), 0x600000000000000n, "{}");
sample.setBigInt64(0, 7n, Symbol("1"));
assert.sameValue(sample.getBigInt64(0), 0x700000000000000n, "symbol");
sample.setBigInt64(0, 8n, 1);
assert.sameValue(sample.getBigInt64(0), 0x800000000000000n, "1");
sample.setBigInt64(0, 9n, "string");
assert.sameValue(sample.getBigInt64(0), 0x900000000000000n, "string");
