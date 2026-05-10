// Copyright (C) 2022 Jordan Harband. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-string.prototype.iswellformed
description: >
  Property type and descriptor.
info: |
  String.prototype.isWellFormed( )

  17 ECMAScript Standard Built-in Objects
includes: [propertyHelper.js]
features: [String.prototype.isWellFormed]
---*/

assert.sameValue(
  typeof String.prototype.isWellFormed,
  'function',
  'The value of `typeof String.prototype.isWellFormed` is "function"'
);

verifyProperty(String.prototype, 'isWellFormed', {
  enumerable: false,
  writable: true,
  configurable: true
});
