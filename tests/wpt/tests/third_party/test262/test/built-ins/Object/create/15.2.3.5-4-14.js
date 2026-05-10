// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.5-4-14
description: >
    Object.create - argument 'Properties' is an Error object (15.2.3.7
    step 2)
---*/

var props = new Error("test");
var result = false;

(Object.getOwnPropertyNames(props)).forEach(function(name) {
  props[name] = {
    value: 11,
    configurable: true
  }
});

Object.defineProperty(props, "prop15_2_3_5_4_14", {
  get: function() {
    result = this instanceof Error;
    return {};
  },
  enumerable: true
});
var newObj = Object.create({}, props);

assert(result, 'result !== true');
assert(newObj.hasOwnProperty("prop15_2_3_5_4_14"), 'newObj.hasOwnProperty("prop15_2_3_5_4_14") !== true');
