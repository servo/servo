// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 26.1.12
description: >
  Reflect.preventExtensions is configurable, writable and not enumerable.
info: |
  26.1.12 Reflect.preventExtensions ( target )

  17 ECMAScript Standard Built-in Objects

includes: [propertyHelper.js]
features: [Reflect]
---*/

verifyProperty(Reflect, 'preventExtensions', {
  writable: true,
  enumerable: false,
  configurable: true
});
