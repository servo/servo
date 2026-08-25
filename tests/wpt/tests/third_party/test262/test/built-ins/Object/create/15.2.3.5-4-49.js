// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.5-4-49
description: >
    Object.create - 'enumerable' property of one property in
    'Properties' is an inherited data property (8.10.5 step 3.a)
---*/

var accessed = false;

var proto = {
  enumerable: true
};
var ConstructFun = function() {};
ConstructFun.prototype = proto;
var descObj = new ConstructFun();

var newObj = Object.create({}, {
  prop: descObj
});

for (var property in newObj) {
  if (property === "prop") {
    accessed = true;
  }
}

assert(accessed, 'accessed !== true');
