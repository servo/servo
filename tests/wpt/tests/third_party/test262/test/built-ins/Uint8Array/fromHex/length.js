// Copyright (C) 2024 Kevin Gibbons. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-uint8array.fromhex
description: >
  Uint8Array.fromHex.length is 1.
includes: [propertyHelper.js]
features: [uint8array-base64, TypedArray]
---*/

verifyProperty(Uint8Array.fromHex, 'length', {
  value: 1,
  enumerable: false,
  writable: false,
  configurable: true
});
