// Copyright (C) 2024 Kevin Gibbons. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-uint8array.prototype.setfrombase64
description: >
  Uint8Array.prototype.setFromBase64 is not a constructor function.
includes: [isConstructor.js]
features: [uint8array-base64, TypedArray, Reflect.construct]
---*/

assert(!isConstructor(Uint8Array.prototype.setFromBase64), "Uint8Array.prototype.setFromBase64 is not a constructor");

assert.throws(TypeError, function() {
  var target = new Uint8Array(10);
  new target.setFromBase64('');
});
