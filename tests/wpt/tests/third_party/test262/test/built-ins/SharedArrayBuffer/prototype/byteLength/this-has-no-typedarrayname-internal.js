// Copyright (C) 2016 the V8 project authors. All rights reserved.
// Copyright (C) 2017 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
description: >
  Throws a TypeError exception when `this` does not have a [[ArrayBufferData]]
  internal slot
features: [SharedArrayBuffer, DataView, Int8Array]
---*/

var getter = Object.getOwnPropertyDescriptor(
  SharedArrayBuffer.prototype, "byteLength"
).get;

assert.throws(TypeError, function() {
  getter.call({});
});

assert.throws(TypeError, function() {
  getter.call([]);
});

var ta = new Int8Array(new SharedArrayBuffer(8));
assert.throws(TypeError, function() {
  getter.call(ta);
});

var dv = new DataView(new SharedArrayBuffer(8), 0);
assert.throws(TypeError, function() {
  getter.call(dv);
});
