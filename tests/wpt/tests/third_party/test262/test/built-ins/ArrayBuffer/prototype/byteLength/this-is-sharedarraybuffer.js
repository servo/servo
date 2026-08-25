// Copyright (C) 2017 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-get-arraybuffer.prototype.bytelength
description: Throws a TypeError exception when `this` is a SharedArrayBuffer
features: [align-detached-buffer-semantics-with-web-reality, SharedArrayBuffer]
---*/

var byteLength = Object.getOwnPropertyDescriptor(
  ArrayBuffer.prototype, "byteLength"
);

var getter = byteLength.get;
var sab = new SharedArrayBuffer(4);

assert.throws(TypeError, function() {
  getter.call(sab);
}, "`this` cannot be a SharedArrayBuffer");

assert.throws(TypeError, function() {
  Object.defineProperties(sab, { byteLength });
  sab.byteLength;
}, "`this` cannot be a SharedArrayBuffer");
