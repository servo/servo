// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: B.2.1
description: >
    Object.getOwnPropertyDescriptor returns data desc for functions on
    built-ins (Global.escape)
includes: [propertyHelper.js]
---*/

assert.sameValue(typeof this.escape, "function");
assert.sameValue(typeof this["escape"], "function");

verifyProperty(this, "escape", {
  writable: true,
  enumerable: false,
  configurable: true
});
