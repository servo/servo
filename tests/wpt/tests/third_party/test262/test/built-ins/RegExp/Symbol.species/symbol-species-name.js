// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 21.2.4.2
description: >
  RegExp[Symbol.species] accessor property get name
info: |
  21.2.4.2 get RegExp [ @@species ]

  ...
  The value of the name property of this function is "get [Symbol.species]".
features: [Symbol.species]
includes: [propertyHelper.js]
---*/

var descriptor = Object.getOwnPropertyDescriptor(RegExp, Symbol.species);

verifyProperty(descriptor.get, "name", {
  value: "get [Symbol.species]",
  writable: false,
  enumerable: false,
  configurable: true
});
