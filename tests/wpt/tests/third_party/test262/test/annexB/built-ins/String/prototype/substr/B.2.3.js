// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: B.2.3
description: >
    Object.getOwnPropertyDescriptor returns data desc for functions on
    built-ins (String.prototype.substr)
includes: [propertyHelper.js]
---*/

verifyProperty(String.prototype, "substr", {
  enumerable: false,
  writable: true,
  configurable: true
});
