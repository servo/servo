// Copyright (C) 2024 Kevin Gibbons. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-uint8array.prototype.tohex
description: >
  Uint8Array.prototype.toHex.length is 0.
includes: [propertyHelper.js]
features: [uint8array-base64, TypedArray]
---*/

verifyProperty(Uint8Array.prototype.toHex, 'length', {
  value: 0,
  enumerable: false,
  writable: false,
  configurable: true
});
