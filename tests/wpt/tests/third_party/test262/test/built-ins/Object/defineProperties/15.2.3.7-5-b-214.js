// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.7-5-b-214
description: >
    Object.defineProperties - 'descObj' is the Arguments object which
    implements its own [[Get]] method to get 'get' property (8.10.5
    step 7.a)
---*/

var obj = {};

var func = function(a, b) {
  arguments.get = function() {
    return "arguments";
  };

  Object.defineProperties(obj, {
    property: arguments
  });

  return obj.property === "arguments";
};

assert(func(), 'func() !== true');
