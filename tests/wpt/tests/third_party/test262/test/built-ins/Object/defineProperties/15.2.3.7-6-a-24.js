// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.7-6-a-24
description: >
    Object.defineProperties - 'O' is the global object which
    implements its own [[GetOwnProperty]] method to get 'P' (8.12.9
    step 1 )
includes: [propertyHelper.js]
---*/


Object.defineProperty(this, "prop", {
  value: 11,
  writable: true,
  enumerable: true,
  configurable: true
});

Object.defineProperties(this, {
  prop: {
    value: 12
  }
});

verifyProperty(this, "prop", {
  value: 12,
  writable: true,
  enumerable: true,
  configurable: true,
});

delete this.prop;
