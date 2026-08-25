// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.7-5-a-8
description: >
    Object.defineProperties - 'Properties' is an Array object which
    implements its own [[Get]] method to get enumerable own property
---*/

var obj = {};
var props = [];
var descObj = {
  value: 8
};

Object.defineProperty(props, "prop", {
  value: descObj,
  enumerable: true
});
Object.defineProperties(obj, props);

assert(obj.hasOwnProperty("prop"), 'obj.hasOwnProperty("prop") !== true');
assert.sameValue(obj.prop, 8, 'obj.prop');
