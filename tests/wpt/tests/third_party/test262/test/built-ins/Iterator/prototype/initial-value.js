// Copyright (C) 2020 Rick Waldron. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-iterator.prototype
description: >
  The initial value of Iterator.prototype is %Iterator.prototype%.
info: |
  Iterator.prototype

  The initial value of Iterator.prototype is %Iterator.prototype%.

  This property has the attributes { [[Writable]]: false, [[Enumerable]]: false, [[Configurable]]: false }.
features: [iterator-helpers]
includes: [propertyHelper.js]
---*/

verifyProperty(Iterator, 'prototype', {
  value: Iterator.prototype,
  writable: false,
  enumerable: false,
  configurable: false,
});
