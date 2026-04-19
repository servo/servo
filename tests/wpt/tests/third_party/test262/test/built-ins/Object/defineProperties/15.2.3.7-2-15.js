// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.7-2-15
description: Object.defineProperties - argument 'Properties' is an Error object
---*/

var obj = {};
var props = new Error("test");
var obj1 = {
  value: 11
};
props.description = obj1;
props.message = obj1;
props.name = obj1;

var result = false;

Object.defineProperty(props, "prop", {
  get: function() {
    result = this instanceof Error;
    return {};
  },
  enumerable: true
});

Object.defineProperties(obj, props);

assert(result, 'result !== true');
