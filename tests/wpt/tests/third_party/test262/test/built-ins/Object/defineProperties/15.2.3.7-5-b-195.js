// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.7-5-b-195
description: >
    Object.defineProperties - 'get' property of 'descObj' is own data
    property that overrides an inherited data property (8.10.5 step
    7.a)
---*/

var obj = {};

var getter = function() {
  return "inheritedDataProperty";
};

var proto = {
  get: getter
};

var Con = function() {};
Con.prototype = proto;

var descObj = new Con();

descObj.get = function() {
  return "ownDataProperty";
};

Object.defineProperties(obj, {
  property: descObj
});

assert.sameValue(obj.property, "ownDataProperty", 'obj.property');
