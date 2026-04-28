// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.7-5-b-227
description: >
    Object.defineProperties - 'set' property of 'descObj' is not
    present (8.10.5 step 8)
includes: [propertyHelper.js]
---*/

var data = "data";
var obj = {};

Object.defineProperties(obj, {
  descObj: {
    get: function() {
      return data;
    }
  }
});


assert(obj.hasOwnProperty("descObj"));
verifyNotWritable(obj, "descObj");

var desc = Object.getOwnPropertyDescriptor(obj, "descObj");
assert.sameValue(typeof(desc.set), "undefined")
