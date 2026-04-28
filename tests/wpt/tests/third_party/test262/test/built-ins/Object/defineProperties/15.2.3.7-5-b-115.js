// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.7-5-b-115
description: >
    Object.defineProperties - 'value' property of 'descObj' is
    inherited data property (8.10.5 step 5.a)
---*/

var obj = {};

var proto = {
  value: "inheritedDataProperty"
};

var Con = function() {};
Con.prototype = proto;

var descObj = new Con();

Object.defineProperties(obj, {
  property: descObj
});

assert.sameValue(obj.property, "inheritedDataProperty", 'obj.property');
