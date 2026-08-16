// Copyright (C) 2025 Kevin Gibbons. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-iterator.prototype.join
description: Iterator.prototype.join.length is 1.
includes: [propertyHelper.js]
features: [Iterator.prototype.join]
---*/

verifyProperty(Iterator.prototype.join, 'length', {
  value: 1,
  enumerable: false,
  writable: false,
  configurable: true
});
