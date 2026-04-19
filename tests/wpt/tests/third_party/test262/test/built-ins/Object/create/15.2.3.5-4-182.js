// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.5-4-182
description: >
    Object.create - 'writable' property of one property in
    'Properties' is own data property that overrides an inherited data
    property (8.10.5 step 6.a)
---*/

var proto = {
  writable: false
};

var ConstructFun = function() {};
ConstructFun.prototype = proto;

var descObj = new ConstructFun();

descObj.writable = true;

var newObj = Object.create({}, {
  prop: descObj
});

var beforeWrite = (newObj.hasOwnProperty("prop") && typeof(newObj.prop) === "undefined");

newObj.prop = "isWritable";

var afterWrite = (newObj.prop === "isWritable");

assert.sameValue(beforeWrite, true, 'beforeWrite');
assert.sameValue(afterWrite, true, 'afterWrite');
