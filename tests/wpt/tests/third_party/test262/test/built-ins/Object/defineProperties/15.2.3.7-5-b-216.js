// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.7-5-b-216
description: >
    Object.defineProperties - 'descObj' is the global object which
    implements its own [[Get]] method to get 'get' property (8.10.5
    step 7.a)
---*/

var obj = {};

this.get = function() {
  return "global";
};

Object.defineProperties(obj, {
  property: this
});

assert.sameValue(obj.property, "global", 'obj.property');
