// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.7-6-a-2
description: >
    Object.defineProperties - 'P' is inherited data property (8.12.9
    step 1 )
includes: [propertyHelper.js]
---*/

var proto = {};
Object.defineProperty(proto, "prop", {
  value: 11,
  configurable: false
});
var Con = function() {};
Con.prototype = proto;

var obj = new Con();

Object.defineProperties(obj, {
  prop: {
    value: 12,
    configurable: true
  }
});

verifyProperty(obj, "prop", {
  value: 12,
  writable: false,
  enumerable: false,
  configurable: true,
});
