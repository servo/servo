// Copyright (C) 2024 Kevin Gibbons. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-uint8array.fromhex
description: >
  Uint8Array.fromHex.name is "fromHex".
includes: [propertyHelper.js]
features: [uint8array-base64, TypedArray]
---*/

verifyProperty(Uint8Array.fromHex, 'name', {
  value: 'fromHex',
  enumerable: false,
  writable: false,
  configurable: true
});
