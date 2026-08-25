// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.7-5-b-228
description: >
    Object.defineProperties - 'set' property of 'descObj' is own data
    property (8.10.5 step 8.a)
---*/

var data = "data";
var obj = {};

Object.defineProperties(obj, {
  descObj: {
    set: function(value) {
      data = value;
    }
  }
});

obj.descObj = "overrideData";

assert(obj.hasOwnProperty("descObj"), 'obj.hasOwnProperty("descObj") !== true');
assert.sameValue(data, "overrideData", 'data');
