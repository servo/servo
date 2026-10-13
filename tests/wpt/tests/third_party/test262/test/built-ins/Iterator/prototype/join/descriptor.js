// Copyright (C) 2025 Kevin Gibbons. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-iterator.prototype.join
description: Iterator.prototype.join has default data property attributes.
includes: [propertyHelper.js]
features: [Iterator.prototype.join]
---*/

verifyProperty(Iterator.prototype, 'join', {
  enumerable: false,
  writable: true,
  configurable: true
});
