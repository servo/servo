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
features: [ArrayBuffer, BigInt, DataView, DataView.prototype.setUint8, Symbol, Symbol.toPrimitive, computed-property-names]
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

assert.throws(RangeError, function() {
  sample.getBigInt64(-1);
}, "ToIndex: throw when integerIndex < 0");
assert.throws(RangeError, function() {
  sample.getBigInt64(-2.5);
}, "ToIndex: throw when integerIndex < 0");
assert.throws(RangeError, function() {
  sample.getBigInt64("-2.5");
}, "ToIndex: parse Number => throw when integerIndex < 0");
assert.throws(RangeError, function() {
  sample.getBigInt64(-Infinity);
}, "ToIndex: throw when integerIndex < 0");
assert.throws(RangeError, function() {
  sample.getBigInt64(9007199254740992);
}, "ToIndex: throw when integerIndex > 2**53-1");
assert.throws(RangeError, function() {
  sample.getBigInt64(Infinity);
}, "ToIndex: throw when integerIndex > 2**53-1");
assert.throws(TypeError, function() {
  sample.getBigInt64(0n);
}, "ToIndex: BigInt => TypeError");
assert.throws(TypeError, function() {
  sample.getBigInt64(Object(0n));
}, "ToIndex: unbox object with internal slot => BigInt => TypeError");
assert.throws(TypeError, function() {
  sample.getBigInt64({
    [Symbol.toPrimitive]: function() {
      return 0n;
    }
  });
}, "ToIndex: @@toPrimitive => BigInt => TypeError");
assert.throws(TypeError, function() {
  sample.getBigInt64({
    valueOf: function() {
      return 0n;
    }
  });
}, "ToIndex: valueOf => BigInt => TypeError");
assert.throws(TypeError, function() {
  sample.getBigInt64({
    toString: function() {
      return 0n;
    }
  });
}, "ToIndex: toString => BigInt => TypeError");
assert.throws(TypeError, function() {
  sample.getBigInt64(Symbol("1"));
}, "ToIndex: Symbol => TypeError");
assert.throws(TypeError, function() {
  sample.getBigInt64(Object(Symbol("1")));
}, "ToIndex: unbox object with internal slot => Symbol => TypeError");
assert.throws(TypeError, function() {
  sample.getBigInt64({
    [Symbol.toPrimitive]: function() {
      return Symbol("1");
    }
  });
}, "ToIndex: @@toPrimitive => Symbol => TypeError");
assert.throws(TypeError, function() {
  sample.getBigInt64({
    valueOf: function() {
      return Symbol("1");
    }
  });
}, "ToIndex: valueOf => Symbol => TypeError");
assert.throws(TypeError, function() {
  sample.getBigInt64({
    toString: function() {
      return Symbol("1");
    }
  });
}, "ToIndex: toString => Symbol => TypeError");
