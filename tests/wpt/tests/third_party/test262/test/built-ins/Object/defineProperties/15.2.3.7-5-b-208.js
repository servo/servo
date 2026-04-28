// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.7-5-b-208
description: >
    Object.defineProperties - 'descObj' is a Number object which
    implements its own [[Get]] method to get 'get' property (8.10.5
    step 7.a)
---*/

var obj = {};

var descObj = new Number(-9);

descObj.get = function() {
  return "Number";
};

Object.defineProperties(obj, {
  property: descObj
});

assert.sameValue(obj.property, "Number", 'obj.property');
