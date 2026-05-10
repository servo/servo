// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: >
    The GeneratorPrototype intrinsic should define a `return` property that is
    non-enumerable, writable, and configurable (as per section 17).
includes: [propertyHelper.js]
es6id: 25.3.1
features: [generators]
---*/

function* g() {}
var GeneratorPrototype = Object.getPrototypeOf(g).prototype;

verifyProperty(GeneratorPrototype, 'return', {
  writable: true,
  enumerable: false,
  configurable: true
});
