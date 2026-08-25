// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-set.prototype.foreach
description: >
    Set.prototype.forEach ( callbackfn [ , thisArg ] )

    The length property of the forEach method is 1.

includes: [propertyHelper.js]
---*/

verifyProperty(Set.prototype.forEach, "length", {
  value: 1,
  writable: false,
  enumerable: false,
  configurable: true
});
