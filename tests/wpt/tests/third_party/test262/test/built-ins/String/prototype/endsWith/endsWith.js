// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 21.1.3.6
description: >
  Property type and descriptor.
info: |
  21.1.3.6 String.prototype.endsWith ( searchString [ , endPosition] )

  17 ECMAScript Standard Built-in Objects
includes: [propertyHelper.js]
features: [String.prototype.endsWith]
---*/

assert.sameValue(
  typeof String.prototype.endsWith,
  'function',
  '`typeof String.prototype.endsWith` is `function`'
);

verifyProperty(String.prototype, 'endsWith', {
  writable: true,
  enumerable: false,
  configurable: true,
});
