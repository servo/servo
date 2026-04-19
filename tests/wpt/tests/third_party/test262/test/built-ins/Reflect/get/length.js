// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 26.1.6
description: >
  Reflect.get.length value and property descriptor
info: |
  26.1.6 Reflect.get ( target, propertyKey [ , receiver ])

  The length property of the get function is 2.
includes: [propertyHelper.js]
features: [Reflect]
---*/

verifyProperty(Reflect.get, "length", {
  value: 2,
  writable: false,
  enumerable: false,
  configurable: true
});
