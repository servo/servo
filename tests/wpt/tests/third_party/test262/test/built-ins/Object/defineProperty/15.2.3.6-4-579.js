// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.6-4-579
description: >
    ES5 Attributes - Success to add property into object (Array
    instance)
---*/

var data = "data";

Object.defineProperty(Array.prototype, "prop", {
  get: function() {
    return data;
  },
  set: function(value) {
    data = value;
  },
  enumerable: true,
  configurable: true
});
var arrObj = [];
arrObj.prop = "myOwnProperty";

assert.sameValue(arrObj.hasOwnProperty("prop"), false, 'arrObj.hasOwnProperty("prop")');
assert.sameValue(arrObj.prop, "myOwnProperty", 'arrObj.prop');
assert.sameValue(data, "myOwnProperty", 'data');
