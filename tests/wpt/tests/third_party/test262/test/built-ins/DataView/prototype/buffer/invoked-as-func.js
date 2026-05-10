// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-get-dataview.prototype.buffer
description: Throws a TypeError exception when invoked as a function
info: |
  24.2.4.1 get DataView.prototype.buffer

  1. Let O be the this value.
  2. If Type(O) is not Object, throw a TypeError exception.
  3. If O does not have a [[DataView]] internal slot, throw a TypeError
  exception.
  ...
---*/

var getter = Object.getOwnPropertyDescriptor(
  DataView.prototype, 'buffer'
).get;

assert.throws(TypeError, function() {
  getter();
});
