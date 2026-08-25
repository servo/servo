// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-get-arraybuffer-@@species
description: >
  ArrayBuffer[Symbol.species] accessor property get name
info: |
  24.1.3.3 get ArrayBuffer [ @@species ]

  ...
  The value of the name property of this function is "get [Symbol.species]".
features: [Symbol.species]
includes: [propertyHelper.js]
---*/

var descriptor = Object.getOwnPropertyDescriptor(ArrayBuffer, Symbol.species);

verifyProperty(descriptor.get, "name", {
  value: "get [Symbol.species]",
  writable: false,
  enumerable: false,
  configurable: true
});
