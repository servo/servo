// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-dataview-buffer-byteoffset-bytelength
description: >
  Throws a TypeError if buffer does not have [[ArrayBufferData]]
info: |
  24.2.2.1 DataView (buffer, byteOffset, byteLength )

  ...
  2. If Type(buffer) is not Object, throw a TypeError exception.
  3. If buffer does not have an [[ArrayBufferData]] internal slot, throw a
  TypeError exception.
  ...
features: [Int8Array]
---*/

var obj = {
  valueOf: function() {
    throw new Test262Error("buffer should be verified before byteOffset");
  }
};

assert.throws(TypeError, function() {
  new DataView({}, obj);
}, "{}");

assert.throws(TypeError, function() {
  new DataView([], obj);
}, "[]");

var ta = new Int8Array();
assert.throws(TypeError, function() {
  new DataView(ta, obj);
}, "typedArray instance");

var other = new DataView(new ArrayBuffer(1), 0);
assert.throws(TypeError, function() {
  new DataView(other, obj);
}, "dataView instance");
