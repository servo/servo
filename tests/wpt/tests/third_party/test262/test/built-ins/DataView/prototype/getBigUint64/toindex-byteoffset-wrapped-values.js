// Copyright (C) 2017 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
description: ToIndex conversions on byteOffset
esid: sec-dataview.prototype.getbiguint64
features: [ArrayBuffer, BigInt, DataView, DataView.prototype.setUint8, Symbol.toPrimitive, computed-property-names]
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

assert.sameValue(sample.getBigUint64(Object(0)), 0x2702060280008001n,
  "ToPrimitive: unbox object with internal slot");
assert.sameValue(sample.getBigUint64({
  [Symbol.toPrimitive]: function() {
    return 0;
  }
}), 0x2702060280008001n, "ToPrimitive: @@toPrimitive");
assert.sameValue(sample.getBigUint64({
  valueOf: function() {
    return 0;
  }
}), 0x2702060280008001n, "ToPrimitive: valueOf");
assert.sameValue(sample.getBigUint64({
  toString: function() {
    return 0;
  }
}), 0x2702060280008001n, "ToPrimitive: toString");
assert.sameValue(sample.getBigUint64(Object(NaN)), 0x2702060280008001n,
  "ToIndex: unbox object with internal slot => NaN => 0");
assert.sameValue(sample.getBigUint64({
  [Symbol.toPrimitive]: function() {
    return NaN;
  }
}), 0x2702060280008001n, "ToIndex: @@toPrimitive => NaN => 0");
assert.sameValue(sample.getBigUint64({
  valueOf: function() {
    return NaN;
  }
}), 0x2702060280008001n, "ToIndex: valueOf => NaN => 0");
assert.sameValue(sample.getBigUint64({
  toString: function() {
    return NaN;
  }
}), 0x2702060280008001n, "ToIndex: toString => NaN => 0");
assert.sameValue(sample.getBigUint64({
  [Symbol.toPrimitive]: function() {
    return undefined;
  }
}), 0x2702060280008001n, "ToIndex: @@toPrimitive => undefined => NaN => 0");
assert.sameValue(sample.getBigUint64({
  valueOf: function() {
    return undefined;
  }
}), 0x2702060280008001n, "ToIndex: valueOf => undefined => NaN => 0");
assert.sameValue(sample.getBigUint64({
  toString: function() {
    return undefined;
  }
}), 0x2702060280008001n, "ToIndex: toString => undefined => NaN => 0");
assert.sameValue(sample.getBigUint64({
  [Symbol.toPrimitive]: function() {
    return null;
  }
}), 0x2702060280008001n, "ToIndex: @@toPrimitive => null => 0");
assert.sameValue(sample.getBigUint64({
  valueOf: function() {
    return null;
  }
}), 0x2702060280008001n, "ToIndex: valueOf => null => 0");
assert.sameValue(sample.getBigUint64({
  toString: function() {
    return null;
  }
}), 0x2702060280008001n, "ToIndex: toString => null => 0");
assert.sameValue(sample.getBigUint64(Object(true)), 0x20602800080017fn,
  "ToIndex: unbox object with internal slot => true => 1");
assert.sameValue(sample.getBigUint64({
  [Symbol.toPrimitive]: function() {
    return true;
  }
}), 0x20602800080017fn, "ToIndex: @@toPrimitive => true => 1");
assert.sameValue(sample.getBigUint64({
  valueOf: function() {
    return true;
  }
}), 0x20602800080017fn, "ToIndex: valueOf => true => 1");
assert.sameValue(sample.getBigUint64({
  toString: function() {
    return true;
  }
}), 0x20602800080017fn, "ToIndex: toString => true => 1");
assert.sameValue(sample.getBigUint64(Object("1")), 0x20602800080017fn,
  "ToIndex: unbox object with internal slot => parse Number");
assert.sameValue(sample.getBigUint64({
  [Symbol.toPrimitive]: function() {
    return "1";
  }
}), 0x20602800080017fn, "ToIndex: @@toPrimitive => parse Number");
assert.sameValue(sample.getBigUint64({
  valueOf: function() {
    return "1";
  }
}), 0x20602800080017fn, "ToIndex: valueOf => parse Number");
assert.sameValue(sample.getBigUint64({
  toString: function() {
    return "1";
  }
}), 0x20602800080017fn, "ToIndex: toString => parse Number");
