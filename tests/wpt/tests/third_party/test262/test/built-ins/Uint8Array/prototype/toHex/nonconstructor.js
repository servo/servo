// Copyright (C) 2024 Kevin Gibbons. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-uint8array.prototype.tohex
description: >
  Uint8Array.prototype.toHex is not a constructor function.
includes: [isConstructor.js]
features: [uint8array-base64, TypedArray, Reflect.construct]
---*/

assert(!isConstructor(Uint8Array.prototype.toHex), "Uint8Array.prototype.toHex is not a constructor");

var uint8Array = new Uint8Array(8);
assert.throws(TypeError, function() {
  new uint8Array.toHex();
});
