// Copyright (C) 2019 Leo Balter. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-string.prototype.replaceall
description: >
  String.prototype.replaceAll.name value and descriptor.
info: |
  String.prototype.replaceAll ( searchValue, replaceValue )

  17 ECMAScript Standard Built-in Objects
includes: [propertyHelper.js]
features: [String.prototype.replaceAll]
---*/

verifyProperty(String.prototype.replaceAll, 'name', {
  value: 'replaceAll',
  enumerable: false,
  writable: false,
  configurable: true,
});
