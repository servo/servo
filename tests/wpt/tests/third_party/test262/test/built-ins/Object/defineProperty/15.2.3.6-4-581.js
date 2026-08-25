// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.6-4-581
description: ES5 Attributes - Fail to add property into object (Number instance)
includes: [propertyHelper.js]
---*/

var data = "data";

Object.defineProperty(Number.prototype, "prop", {
  get: function() {
    return data;
  },
  enumerable: false,
  configurable: true
});
var numObj = new Number();

verifyNotWritable(numObj, "prop", "nocheck");

assert(!numObj.hasOwnProperty("prop"));
assert.sameValue(numObj.prop, "data");
assert.sameValue(data, "data");
