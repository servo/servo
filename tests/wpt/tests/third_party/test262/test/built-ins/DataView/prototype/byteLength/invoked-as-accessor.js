// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-get-dataview.prototype.bytelength
description: >
  Requires this value to have a [[DataView]] internal slot
info: |
  24.2.4.2 get DataView.prototype.byteLength

  1. Let O be the this value.
  2. If Type(O) is not Object, throw a TypeError exception.
  3. If O does not have a [[DataView]] internal slot, throw a TypeError
  exception.
  ...
---*/

assert.throws(TypeError, function() {
  DataView.prototype.byteLength;
});
