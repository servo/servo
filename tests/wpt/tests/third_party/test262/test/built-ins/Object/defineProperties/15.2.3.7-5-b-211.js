// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.7-5-b-211
description: >
    Object.defineProperties - 'descObj' is a RegExp object which
    implements its own [[Get]] method to get 'get' property (8.10.5
    step 7.a)
---*/

var obj = {};

var descObj = new RegExp();

descObj.get = function() {
  return "RegExp";
};

Object.defineProperties(obj, {
  property: descObj
});

assert.sameValue(obj.property, "RegExp", 'obj.property');
