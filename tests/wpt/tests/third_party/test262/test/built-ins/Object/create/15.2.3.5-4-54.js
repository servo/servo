// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.5-4-54
description: >
    Object.create - 'enumerable' property of one property in
    'Properties' is own accessor property that overrides an inherited
    data property (8.10.5 step 3.a)
---*/

var accessed = false;
var proto = {
  enumerable: true
};

var ConstructFun = function() {};
ConstructFun.prototype = proto;
var descObj = new ConstructFun();

Object.defineProperty(descObj, "enumerable", {
  get: function() {
    return false;
  }
});

var newObj = Object.create({}, {
  prop: descObj
});
for (var property in newObj) {
  if (property === "prop") {
    accessed = true;
  }
}

assert.sameValue(accessed, false, 'accessed');
