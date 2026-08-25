// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.6-4-596
description: >
    ES5 Attributes - Fail to update value of property into of
    [[Proptotype]] internal property (Function.prototype.bind)
includes: [propertyHelper.js]
---*/

var foo = function() {};
var data = "data";

Object.defineProperty(Function.prototype, "prop", {
  get: function() {
    return data;
  },
  enumerable: false,
  configurable: true
});

var obj = foo.bind({});

assert(!obj.hasOwnProperty("prop"));
verifyNotWritable(obj, "prop", "nocheck");
assert.sameValue(obj.prop, "data");;
