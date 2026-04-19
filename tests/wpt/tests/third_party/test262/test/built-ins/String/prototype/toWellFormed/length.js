// Copyright (C) 2022 Jordan Harband. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-string.prototype.towellformed
description: >
  String.prototype.toWellFormed.length value and descriptor.
info: |
  String.prototype.toWellFormed( )

  17 ECMAScript Standard Built-in Objects

includes: [propertyHelper.js]
features: [String.prototype.toWellFormed]
---*/

verifyProperty(String.prototype.toWellFormed, 'length', {
  configurable: true,
  enumerable: false,
  writable: false,
  value: 0
});
