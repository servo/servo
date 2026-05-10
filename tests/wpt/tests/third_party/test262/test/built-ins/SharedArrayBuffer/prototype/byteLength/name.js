// Copyright (C) 2015 the V8 project authors. All rights reserved.
// Copyright (C) 2017 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
description: >
  get SharedArrayBuffer.prototype.byteLength

includes: [propertyHelper.js]
features: [SharedArrayBuffer]
---*/

var descriptor = Object.getOwnPropertyDescriptor(
  SharedArrayBuffer.prototype, 'byteLength'
);

verifyProperty(descriptor.get, "name", {
  value: "get byteLength",
  writable: false,
  enumerable: false,
  configurable: true
});
