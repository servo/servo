// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.7-5-b-152
description: >
    Object.defineProperties - 'descObj' is an Array object which
    implements its own [[Get]] method to get 'writable' property
    (8.10.5 step 6.a)
includes: [propertyHelper.js]
---*/

var obj = {};

var arr = [1, 2, 3];

arr.writable = false;

Object.defineProperties(obj, {
  property: arr
});

verifyProperty(obj, "property", {
  writable: false,
});
