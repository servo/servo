// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.7-5-b-210
description: >
    Object.defineProperties - 'descObj' is a Date object which
    implements its own [[Get]] method to get 'get' property (8.10.5
    step 7.a)
---*/

var obj = {};

var descObj = new Date(0);

descObj.get = function() {
  return "Date";
};

Object.defineProperties(obj, {
  property: descObj
});

assert.sameValue(obj.property, "Date", 'obj.property');
