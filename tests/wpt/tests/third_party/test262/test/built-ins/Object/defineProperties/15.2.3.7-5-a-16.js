// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.7-5-a-16
description: >
    Object.defineProperties - 'Properties' is an Error object which
    implements its own [[Get]] method to get enumerable own property
---*/

var obj = {};
var props = new Error("test");
var obj1 = {
  value: 11
};
props.message = obj1;
props.name = obj1;
props.description = obj1;

props.prop = {
  value: 16
};
Object.defineProperties(obj, props);

assert(obj.hasOwnProperty("prop"), 'obj.hasOwnProperty("prop") !== true');
assert.sameValue(obj.prop, 16, 'obj.prop');
