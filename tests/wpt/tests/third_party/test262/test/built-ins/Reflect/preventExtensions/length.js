// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 26.1.12
description: >
  Reflect.preventExtensions.length value and property descriptor
includes: [propertyHelper.js]
features: [Reflect]
---*/

verifyProperty(Reflect.preventExtensions, "length", {
  value: 1,
  writable: false,
  enumerable: false,
  configurable: true
});
