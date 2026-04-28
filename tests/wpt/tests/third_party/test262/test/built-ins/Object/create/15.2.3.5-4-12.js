// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.5-4-12
description: >
    Object.create - argument 'Properties' is a RegExp object (15.2.3.7
    step 2)
---*/

var props = new RegExp();
var result = false;

Object.defineProperty(props, "prop", {
  get: function() {
    result = this instanceof RegExp;
    return {};
  },
  enumerable: true
});
var newObj = Object.create({}, props);

assert(result, 'result !== true');
assert(newObj.hasOwnProperty("prop"), 'newObj.hasOwnProperty("prop") !== true');
