// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.7-5-b-226
description: >
    Object.defineProperties - 'set' property of 'descObj' is present
    (8.10.5 step 8)
---*/

var data = "data";
var obj = {};

Object.defineProperties(obj, {
  "prop": {
    set: function(value) {
      data = value;
    }
  }
});

obj.prop = "overrideData";

assert(obj.hasOwnProperty("prop"), 'obj.hasOwnProperty("prop") !== true');
assert.sameValue(data, "overrideData", 'data');
