// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: B.2.6
description: >
    Object.getOwnPropertyDescriptor returns data desc for functions on
    built-ins (Date.prototype.toGMTString)
includes: [propertyHelper.js]

---*/

verifyProperty(Date.prototype, "toGMTString", {
  enumerable: false,
  writable: true,
  configurable: true,
});
