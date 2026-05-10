// Copyright (C) 2024 Kevin Gibbons. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-uint8array.prototype.tohex
description: >
  Uint8Array.prototype.toHex.name is "toHex".
includes: [propertyHelper.js]
features: [uint8array-base64, TypedArray]
---*/

verifyProperty(Uint8Array.prototype.toHex, 'name', {
  value: 'toHex',
  enumerable: false,
  writable: false,
  configurable: true
});
