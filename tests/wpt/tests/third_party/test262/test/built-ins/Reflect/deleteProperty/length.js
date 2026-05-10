// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 26.1.4
description: >
  Reflect.deleteProperty.length value and property descriptor
includes: [propertyHelper.js]
features: [Reflect]
---*/

verifyProperty(Reflect.deleteProperty, "length", {
  value: 2,
  writable: false,
  enumerable: false,
  configurable: true
});
