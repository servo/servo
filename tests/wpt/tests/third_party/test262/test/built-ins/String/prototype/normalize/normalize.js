// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 21.1.3.12
description: >
  Property type and descriptor.
info: |
  21.1.3.12 String.prototype.normalize ( [ form ] )

  17 ECMAScript Standard Built-in Objects
includes: [propertyHelper.js]
---*/

assert.sameValue(
  typeof String.prototype.normalize,
  'function',
  '`typeof String.prototype.normalize` is `function`'
);

verifyProperty(String.prototype, 'normalize', {
  writable: true,
  enumerable: false,
  configurable: true,
});
