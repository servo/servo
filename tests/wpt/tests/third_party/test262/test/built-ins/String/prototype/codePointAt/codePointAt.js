// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 21.1.3.3
description: >
  Property type and descriptor.
info: |
  21.1.3.3 String.prototype.codePointAt ( pos )

  17 ECMAScript Standard Built-in Objects
includes: [propertyHelper.js]
---*/

assert.sameValue(
  typeof String.prototype.codePointAt,
  'function',
  '`typeof String.prototype.codePointAt` is `function`'
);

verifyProperty(String.prototype, 'codePointAt', {
  writable: true,
  enumerable: false,
  configurable: true,
});
