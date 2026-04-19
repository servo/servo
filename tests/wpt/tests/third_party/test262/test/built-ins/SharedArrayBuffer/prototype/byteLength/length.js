// Copyright (C) 2015 André Bargull. All rights reserved.
// Copyright (C) 2017 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: >
  get SharedArrayBuffer.prototype.byteLength.length is 0.
includes: [propertyHelper.js]
features: [SharedArrayBuffer]
---*/

var desc = Object.getOwnPropertyDescriptor(SharedArrayBuffer.prototype, "byteLength");

verifyProperty(desc.get, "length", {
  value: 0,
  writable: false,
  enumerable: false,
  configurable: true
});
