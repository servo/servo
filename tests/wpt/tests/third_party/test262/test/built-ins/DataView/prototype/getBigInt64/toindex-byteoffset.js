// Copyright (C) 2017 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
description: ToIndex conversions on byteOffset
esid: sec-dataview.prototype.getbigint64
info: |
  DataView.prototype.getBigInt64 ( byteOffset [ , littleEndian ] )

  1. Let v be the this value.
  2. If littleEndian is not present, let littleEndian be undefined.
  3. Return ? GetViewValue(v, byteOffset, littleEndian, "Int64").

  24.3.1.1 GetViewValue ( view, requestIndex, isLittleEndian, type )

  ...
  4. Let getIndex be ? ToIndex(requestIndex).
  ...
features: [ArrayBuffer, BigInt, DataView, DataView.prototype.setUint8]
---*/

var buffer = new ArrayBuffer(12);
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

assert.sameValue(sample.getBigInt64(0), 0x2702060280008001n);
assert.sameValue(sample.getBigInt64(1), 0x20602800080017fn);
assert.sameValue(sample.getBigInt64(-0.9), 0x2702060280008001n, "ToIndex: truncate towards 0");
assert.sameValue(sample.getBigInt64(0.9), 0x2702060280008001n, "ToIndex: truncate towards 0");
assert.sameValue(sample.getBigInt64(NaN), 0x2702060280008001n, "ToIndex: NaN => 0");
assert.sameValue(sample.getBigInt64(undefined), 0x2702060280008001n,
  "ToIndex: undefined => NaN => 0");
assert.sameValue(sample.getBigInt64(null), 0x2702060280008001n, "ToIndex: null => 0");
assert.sameValue(sample.getBigInt64(false), 0x2702060280008001n, "ToIndex: false => 0");
assert.sameValue(sample.getBigInt64(true), 0x20602800080017fn, "ToIndex: true => 1");
assert.sameValue(sample.getBigInt64("0"), 0x2702060280008001n, "ToIndex: parse Number");
assert.sameValue(sample.getBigInt64("1"), 0x20602800080017fn, "ToIndex: parse Number");
assert.sameValue(sample.getBigInt64(""), 0x2702060280008001n, "ToIndex: parse Number => NaN => 0");
assert.sameValue(sample.getBigInt64("foo"), 0x2702060280008001n,
  "ToIndex: parse Number => NaN => 0");
assert.sameValue(sample.getBigInt64("true"), 0x2702060280008001n,
  "ToIndex: parse Number => NaN => 0");
assert.sameValue(sample.getBigInt64(2), 0x602800080017F00n);
assert.sameValue(sample.getBigInt64("2"), 0x602800080017F00n, "toIndex: parse Number");
assert.sameValue(sample.getBigInt64(2.9), 0x602800080017F00n, "toIndex: truncate towards 0");
assert.sameValue(sample.getBigInt64("2.9"), 0x602800080017F00n,
  "toIndex: parse Number => truncate towards 0");
assert.sameValue(sample.getBigInt64(3), 0x2800080017F0001n);
assert.sameValue(sample.getBigInt64("3"), 0x2800080017F0001n, "toIndex: parse Number");
assert.sameValue(sample.getBigInt64(3.9), 0x2800080017F0001n, "toIndex: truncate towards 0");
assert.sameValue(sample.getBigInt64("3.9"), 0x2800080017F0001n,
  "toIndex: parse Number => truncate towards 0");
assert.sameValue(sample.getBigInt64([0]), 0x2702060280008001n,
  'ToIndex: [0].toString() => "0" => 0');
assert.sameValue(sample.getBigInt64(["1"]), 0x20602800080017fn,
  'ToIndex: ["1"].toString() => "1" => 1');
assert.sameValue(sample.getBigInt64({}), 0x2702060280008001n,
  'ToIndex: ({}).toString() => "[object Object]" => NaN => 0');
assert.sameValue(sample.getBigInt64([]), 0x2702060280008001n,
  'ToIndex: [].toString() => "" => NaN => 0');
