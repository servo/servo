// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.6-4-540-7
description: >
    Object.defineProperty fails to update [[Get]] and [[Set]]
    attributes of a named property 'P' whose [[Configurable]]
    attribute is false and throws TypeError exception, 'A' is an Array
    object (8.12.9 step 11.a)
includes: [propertyHelper.js]
---*/

var obj = [];

obj.verifySetFunction = "data";
var getFunc = function() {
  return obj.verifySetFunction;
};
var setFunc = function(value) {
  obj.verifySetFunction = value;
};
Object.defineProperty(obj, "prop", {
  get: getFunc,
  set: setFunc,
  configurable: false
});

var result = false;
try {
  Object.defineProperty(obj, "prop", {
    get: function() {
      return 100;
    }
  });
} catch (e) {
  result = e instanceof TypeError;
  verifyEqualTo(obj, "prop", getFunc());

  verifyWritable(obj, "prop", "verifySetFunction");
}

verifyProperty(obj, "prop", {
  enumerable: false,
  configurable: false,
});

try {
  Object.defineProperty(obj, "prop", {
    set: function(value) {
      obj.verifySetFunction1 = value;
    }
  });
} catch (e1) {
  if (!result) {
    throw new Test262Error('Expected result to be true, actually ' + result);
  }


  verifyEqualTo(obj, "prop", getFunc());

  verifyWritable(obj, "prop", "verifySetFunction");

  if (!(e1 instanceof TypeError)) {
    throw new Test262Error("Expected TypeError, got " + e1);
  }
}

verifyProperty(obj, "prop", {
  enumerable: false,
  configurable: false,
});
