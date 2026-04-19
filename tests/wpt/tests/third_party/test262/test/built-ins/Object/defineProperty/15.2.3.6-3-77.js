// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.6-3-77
description: >
    Object.defineProperty - 'configurable' property in 'Attributes' is
    own data property that overrides an inherited data property
    (8.10.5 step 4.a)
---*/

var obj = {};

var proto = {
  configurable: false
};

var ConstructFun = function() {};
ConstructFun.prototype = proto;

var child = new ConstructFun();
child.configurable = true;

Object.defineProperty(obj, "property", child);

var beforeDeleted = obj.hasOwnProperty("property");

delete obj.property;

var afterDeleted = obj.hasOwnProperty("property");

assert.sameValue(beforeDeleted, true, 'beforeDeleted');
assert.sameValue(afterDeleted, false, 'afterDeleted');
