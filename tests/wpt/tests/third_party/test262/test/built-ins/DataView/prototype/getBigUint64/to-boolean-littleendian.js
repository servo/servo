// Copyright (C) 2017 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
description: Boolean littleEndian argument coerced in ToBoolean
esid: sec-dataview.prototype.getbiguint64
features: [ArrayBuffer, BigInt, DataView, DataView.prototype.setUint8, Symbol]
---*/

var buffer = new ArrayBuffer(8);
var sample = new DataView(buffer, 0);
sample.setUint8(7, 0xff);
assert.sameValue(sample.getBigUint64(0), 0xffn, "no argument");

assert.sameValue(sample.getBigUint64(0, false), 0xffn);
assert.sameValue(sample.getBigUint64(0, true), 0xff00000000000000n);
assert.sameValue(sample.getBigUint64(0, 0), 0xffn, "ToBoolean: 0 => false");
assert.sameValue(sample.getBigUint64(0, -0), 0xffn, "ToBoolean: -0 => false");
assert.sameValue(sample.getBigUint64(0, 1), 0xff00000000000000n, "ToBoolean: Number != 0 => true");
assert.sameValue(sample.getBigUint64(0, -1), 0xff00000000000000n, "ToBoolean: Number != 0 => true");
assert.sameValue(sample.getBigUint64(0, 0.1), 0xff00000000000000n, "ToBoolean: Number != 0 => true");
assert.sameValue(sample.getBigUint64(0, Infinity), 0xff00000000000000n,
  "ToBoolean: Number != 0 => true");
assert.sameValue(sample.getBigUint64(0, NaN), 0xffn, "ToBoolean: NaN => false");
assert.sameValue(sample.getBigUint64(0, undefined), 0xffn, "ToBoolean: undefined => false");
assert.sameValue(sample.getBigUint64(0, null), 0xffn, "ToBoolean: null => false");
assert.sameValue(sample.getBigUint64(0, ""), 0xffn, "ToBoolean: String .length == 0 => false");
assert.sameValue(sample.getBigUint64(0, "string"), 0xff00000000000000n,
  "ToBoolean: String .length > 0 => true");
assert.sameValue(sample.getBigUint64(0, "false"), 0xff00000000000000n,
  "ToBoolean: String .length > 0 => true");
assert.sameValue(sample.getBigUint64(0, " "), 0xff00000000000000n,
  "ToBoolean: String .length > 0 => true");
assert.sameValue(sample.getBigUint64(0, Symbol("1")), 0xff00000000000000n,
  "ToBoolean: Symbol => true");
assert.sameValue(sample.getBigUint64(0, 0n), 0xffn, "ToBoolean: 0n => false");
assert.sameValue(sample.getBigUint64(0, 1n), 0xff00000000000000n, "ToBoolean: BigInt != 0n => true");
assert.sameValue(sample.getBigUint64(0, []), 0xff00000000000000n, "ToBoolean: any object => true");
assert.sameValue(sample.getBigUint64(0, {}), 0xff00000000000000n, "ToBoolean: any object => true");
assert.sameValue(sample.getBigUint64(0, Object(false)), 0xff00000000000000n,
  "ToBoolean: any object => true; no ToPrimitive");
