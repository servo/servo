// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-array.prototype.copywithin
description: Array.prototype.copyWithin.length value and descriptor.
info: |
  22.1.3.3 Array.prototype.copyWithin (target, start [ , end ] )

  The length property of the copyWithin method is 2.
includes: [propertyHelper.js]
---*/

verifyProperty(Array.prototype.copyWithin, "length", {
  value: 2,
  writable: false,
  enumerable: false,
  configurable: true
});
