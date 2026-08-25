// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.7-6-a-17
description: >
    Object.defineProperties - 'O' is the Math object which implements
    its own [[GetOwnProperty]] method to get 'P' (8.12.9 step 1 )
---*/

Object.defineProperty(Math, "prop", {
  value: 11,
  writable: true,
  configurable: true
});
var hasProperty = Math.hasOwnProperty("prop") && Math.prop === 11;

Object.defineProperties(Math, {
  prop: {
    value: 12
  }
});

assert(hasProperty, 'hasProperty !== true');
assert.sameValue(Math.prop, 12, 'Math.prop');
