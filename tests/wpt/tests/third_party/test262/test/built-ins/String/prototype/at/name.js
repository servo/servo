// Copyright (C) 2020 Rick Waldron. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-string.prototype.at
description: >
  String.prototype.at.name value and descriptor.
info: |
  String.prototype.at( index )

  17 ECMAScript Standard Built-in Objects

includes: [propertyHelper.js]
features: [String.prototype.at]
---*/
assert.sameValue(typeof String.prototype.at, 'function');

assert.sameValue(
  String.prototype.at.name, 'at',
  'The value of String.prototype.at.name is "at"'
);

verifyProperty(String.prototype.at, 'name', {
  enumerable: false,
  writable: false,
  configurable: true
});
