// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-dataview.prototype.getfloat64
description: >
  ToIndex conversions on byteOffset
info: |
  24.2.4.6 DataView.prototype.getFloat64 ( byteOffset [ , littleEndian ] )

  1. Let v be the this value.
  2. If littleEndian is not present, let littleEndian be false.
  3. Return ? GetViewValue(v, byteOffset, littleEndian, "Float64").

  24.2.1.1 GetViewValue ( view, requestIndex, isLittleEndian, type )

  ...
  4. Let getIndex be ? ToIndex(requestIndex).
  ...
features: [DataView.prototype.setUint8]
---*/

var buffer = new ArrayBuffer(12);
var sample = new DataView(buffer, 0);

sample.setUint8(0, 67);
sample.setUint8(1, 67);
sample.setUint8(2, 68);
sample.setUint8(3, 68);
sample.setUint8(4, 67);
sample.setUint8(5, 67);
sample.setUint8(6, 68);
sample.setUint8(7, 68);
sample.setUint8(8, 67);
sample.setUint8(9, 68);
sample.setUint8(10, 68);
sample.setUint8(11, 68);

var obj1 = {
  valueOf: function() {
    return 2;
  }
};

var obj2 = {
  toString: function() {
    return 3;
  }
};

assert.sameValue(sample.getFloat64(-0), 10846169068898440, "-0");
assert.sameValue(sample.getFloat64(obj1), 747563348316297600000, "{}.valueOf");
assert.sameValue(sample.getFloat64(obj2), 710670423110275600000, "{}.toString");
assert.sameValue(sample.getFloat64(""), 10846169068898440, "the Empty string");
assert.sameValue(sample.getFloat64("0"), 10846169068898440, "string '0'");
assert.sameValue(sample.getFloat64("2"), 747563348316297600000, "string '2'");
assert.sameValue(sample.getFloat64(true), 11409110432516230, "true");
assert.sameValue(sample.getFloat64(false), 10846169068898440, "false");
assert.sameValue(sample.getFloat64(NaN), 10846169068898440, "NaN");
assert.sameValue(sample.getFloat64(null), 10846169068898440, "null");
assert.sameValue(sample.getFloat64(0.1), 10846169068898440, "0.1");
assert.sameValue(sample.getFloat64(0.9), 10846169068898440, "0.9");
assert.sameValue(sample.getFloat64(1.1), 11409110432516230, "1.1");
assert.sameValue(sample.getFloat64(1.9), 11409110432516230, "1.9");
assert.sameValue(sample.getFloat64(-0.1), 10846169068898440, "-0.1");
assert.sameValue(sample.getFloat64(-0.99999), 10846169068898440, "-0.99999");
assert.sameValue(sample.getFloat64(undefined), 10846169068898440, "undefined");
assert.sameValue(sample.getFloat64(), 10846169068898440, "no arg");
