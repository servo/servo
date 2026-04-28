// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.7-5-b-161
description: >
    Object.defineProperties - 'descObj' is the Arguments object which
    implements its own [[Get]] method to get 'writable' property
    (8.10.5 step 6.a)
includes: [propertyHelper.js]
---*/

var obj = {};

var func = function(a, b) {
  arguments.writable = false;

  Object.defineProperties(obj, {
    property: arguments
  });

  verifyProperty(obj, "property", {
    writable: false,
  });
};

func();
