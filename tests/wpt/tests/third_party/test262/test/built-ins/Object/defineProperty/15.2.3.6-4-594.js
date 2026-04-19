// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.6-4-594
description: >
    ES5 Attributes - Success to update value of property into of
    [[Proptotype]] internal property (Function.prototype.bind)
---*/

var foo = function() {};
var data = "data";

Object.defineProperty(Function.prototype, "prop", {
  get: function() {
    return data;
  },
  set: function(value) {
    data = value;
  },
  enumerable: true,
  configurable: true
});

var obj = foo.bind({});
obj.prop = "overrideData";

assert.sameValue(obj.hasOwnProperty("prop"), false, 'obj.hasOwnProperty("prop")');
assert.sameValue(obj.prop, "overrideData", 'obj.prop');
