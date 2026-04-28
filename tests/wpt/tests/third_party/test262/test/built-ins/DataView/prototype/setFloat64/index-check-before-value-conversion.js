// Copyright (C) 2015 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-dataview.prototype.setfloat64
description: >
  RangeError exception for negative or non-integral index is thrown before
  the value conversion.
info: |
  ...
  3. Return SetViewValue(v, byteOffset, littleEndian, "Float64", value).

  24.2.1.2 SetViewValue ( view, requestIndex, isLittleEndian, type, value )

  ...
  4. Let getIndex be ? ToIndex(requestIndex).
  ...
---*/

var dataView = new DataView(new ArrayBuffer(16), 0);

var poisoned = {
  valueOf: function() {
    throw new Test262Error("valueOf called");
  }
};

assert.throws(RangeError, function() {
  dataView.setFloat64(-1.5, poisoned);
}, "setFloat64(-1.5, poisoned)");

assert.throws(RangeError, function() {
  dataView.setFloat64(-1, poisoned);
}, "setFloat64(-1, poisoned)");

assert.throws(RangeError, function() {
  dataView.setFloat64(-Infinity, poisoned);
}, "setFloat64(-Infinity, poisoned)");

assert.throws(RangeError, function() {
  dataView.setFloat64(Infinity, poisoned);
}, "setFloat64(Infinity, poisoned)");
