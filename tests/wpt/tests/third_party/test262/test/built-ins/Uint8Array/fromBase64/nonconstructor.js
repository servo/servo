// Copyright (C) 2024 Kevin Gibbons. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-uint8array.frombase64
description: >
  Uint8Array.fromBase64 is not a constructor function.
includes: [isConstructor.js]
features: [uint8array-base64, TypedArray, Reflect.construct]
---*/

assert(!isConstructor(Uint8Array.fromBase64), "Uint8Array.fromBase64 is not a constructor");

assert.throws(TypeError, function() {
  new Uint8Array.fromBase64('');
});
