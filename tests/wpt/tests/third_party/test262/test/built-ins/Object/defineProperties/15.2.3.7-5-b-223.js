// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.7-5-b-223
description: >
    Object.defineProperties - value of 'get' property of 'descObj' is
    a function (8.10.5 step 7.b)
---*/

var obj = {};

var getter = function() {
  return 100;
};

Object.defineProperties(obj, {
  property: {
    get: getter
  }
});

assert.sameValue(obj.property, 100, 'obj.property');
