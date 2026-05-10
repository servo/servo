// Copyright (C) 2017 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
description: Boolean littleEndian argument coerced in ToBoolean
esid: sec-dataview.prototype.getbigint64
info: |
  DataView.prototype.getBigInt64 ( byteOffset [ , littleEndian ] )

  1. Let v be the this value.
  2. If littleEndian is not present, let littleEndian be undefined.
  3. Return ? GetViewValue(v, byteOffset, littleEndian, "Int64").

  24.3.1.1 GetViewValue ( view, requestIndex, isLittleEndian, type )

  ...
  5. Set isLittleEndian to ToBoolean(isLittleEndian).
  ...
  12. Let bufferIndex be getIndex + viewOffset.
  13. Return GetValueFromBuffer(buffer, bufferIndex, type, false,
  "Unordered", isLittleEndian).

  24.1.1.6 GetValueFromBuffer ( arrayBuffer, byteIndex, type,
  isTypedArray, order [ , isLittleEndian ] )

  ...
  9. Return RawBytesToNumber(type, rawValue, isLittleEndian).

  24.1.1.5 RawBytesToNumber( type, rawBytes, isLittleEndian )

  ...
  2. If isLittleEndian is false, reverse the order of the elements of rawBytes.
  ...
features: [ArrayBuffer, BigInt, DataView, DataView.prototype.setUint8, Symbol]
---*/

var buffer = new ArrayBuffer(8);
var sample = new DataView(buffer, 0);
sample.setUint8(7, 0xff);
assert.sameValue(sample.getBigInt64(0), 0xffn, "no argument");

assert.sameValue(sample.getBigInt64(0, false), 0xffn);
assert.sameValue(sample.getBigInt64(0, true), -0x100000000000000n);
assert.sameValue(sample.getBigInt64(0, 0), 0xffn, "ToBoolean: 0 => false");
assert.sameValue(sample.getBigInt64(0, -0), 0xffn, "ToBoolean: -0 => false");
assert.sameValue(sample.getBigInt64(0, 1), -0x100000000000000n, "ToBoolean: Number != 0 => true");
assert.sameValue(sample.getBigInt64(0, -1), -0x100000000000000n, "ToBoolean: Number != 0 => true");
assert.sameValue(sample.getBigInt64(0, 0.1), -0x100000000000000n, "ToBoolean: Number != 0 => true");
assert.sameValue(sample.getBigInt64(0, Infinity), -0x100000000000000n,
  "ToBoolean: Number != 0 => true");
assert.sameValue(sample.getBigInt64(0, NaN), 0xffn, "ToBoolean: NaN => false");
assert.sameValue(sample.getBigInt64(0, undefined), 0xffn, "ToBoolean: undefined => false");
assert.sameValue(sample.getBigInt64(0, null), 0xffn, "ToBoolean: null => false");
assert.sameValue(sample.getBigInt64(0, ""), 0xffn, "ToBoolean: String .length == 0 => false");
assert.sameValue(sample.getBigInt64(0, "string"), -0x100000000000000n,
  "ToBoolean: String .length > 0 => true");
assert.sameValue(sample.getBigInt64(0, "false"), -0x100000000000000n,
  "ToBoolean: String .length > 0 => true");
assert.sameValue(sample.getBigInt64(0, " "), -0x100000000000000n,
  "ToBoolean: String .length > 0 => true");
assert.sameValue(sample.getBigInt64(0, Symbol("1")), -0x100000000000000n,
  "ToBoolean: Symbol => true");
assert.sameValue(sample.getBigInt64(0, 0n), 0xffn, "ToBoolean: 0n => false");
assert.sameValue(sample.getBigInt64(0, 1n), -0x100000000000000n, "ToBoolean: BigInt != 0n => true");
assert.sameValue(sample.getBigInt64(0, []), -0x100000000000000n, "ToBoolean: any object => true");
assert.sameValue(sample.getBigInt64(0, {}), -0x100000000000000n, "ToBoolean: any object => true");
assert.sameValue(sample.getBigInt64(0, Object(false)), -0x100000000000000n,
  "ToBoolean: any object => true; no ToPrimitive");
