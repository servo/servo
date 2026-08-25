// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.5-4-103
description: >
    Object.create - 'configurable' property of one property in
    'Properties' is own data property that overrides an inherited data
    property (8.10.5 step 4.a)
includes: [propertyHelper.js]
---*/

var proto = {
  configurable: true
};

var ConstructFun = function() {};
ConstructFun.prototype = proto;
var descObj = new ConstructFun();

Object.defineProperty(descObj, "configurable", {
  value: false
});

var newObj = Object.create({}, {
  prop: descObj
});

verifyProperty(newObj, "prop", {
  configurable: false,
});
