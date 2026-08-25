// Copyright (C) 2020 Rick Waldron. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-string.prototype.at
description: >
  Property type and descriptor.
info: |
  String.prototype.at( index )

  17 ECMAScript Standard Built-in Objects
includes: [propertyHelper.js]
features: [String.prototype.at]
---*/
assert.sameValue(typeof String.prototype.at, 'function');

assert.sameValue(
  typeof String.prototype.at,
  'function',
  'The value of `typeof String.prototype.at` is "function"'
);

verifyProperty(String.prototype, 'at', {
  enumerable: false,
  writable: true,
  configurable: true
});
