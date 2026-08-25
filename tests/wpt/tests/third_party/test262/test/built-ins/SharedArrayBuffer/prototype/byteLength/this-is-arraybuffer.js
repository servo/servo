// Copyright (C) 2017 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-get-sharedarraybuffer.prototype.bytelength
description: Throws a TypeError exception when `this` is an ArrayBuffer
features: [SharedArrayBuffer]
---*/

var getter = Object.getOwnPropertyDescriptor(
  SharedArrayBuffer.prototype, "byteLength"
).get;

assert.throws(TypeError, function() {
  var ab = new ArrayBuffer(4);
  getter.call(ab);
}, "`this` cannot be an ArrayBuffer");
