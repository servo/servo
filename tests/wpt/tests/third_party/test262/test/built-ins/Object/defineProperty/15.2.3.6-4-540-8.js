// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.6-4-540-8
description: >
    Object.defineProperty fails to update [[Get]] and [[Set]]
    attributes of an indexed accessor property 'P' whose
    [[Configurable]] attribute is false, 'O' is an Arguments object
    (8.12.9 step 11.a)
includes: [propertyHelper.js]
---*/

var obj = (function() {
  return arguments;
}());

obj.verifySetFunction = "data";
var getFunc = function() {
  return obj.verifySetFunction;
};
var setFunc = function(value) {
  obj.verifySetFunction = value;
};
Object.defineProperty(obj, "0", {
  get: getFunc,
  set: setFunc,
  configurable: false
});

var result = false;
try {
  Object.defineProperty(obj, "0", {
    get: function() {
      return 100;
    }
  });
} catch (e) {
  result = e instanceof TypeError;
  verifyEqualTo(obj, "0", getFunc());

  verifyWritable(obj, "0", "verifySetFunction");
}

verifyProperty(obj, "0", {
  enumerable: false,
  configurable: false,
});

try {
  Object.defineProperty(obj, "0", {
    set: function(value) {
      obj.verifySetFunction1 = value;
    }
  });
} catch (e) {
  if (!result) {
    throw new Test262Error('Expected result  to be true, actually ' + result);
  }

  verifyEqualTo(obj, "0", getFunc());

  verifyWritable(obj, "0", "verifySetFunction");

  if (!(e instanceof TypeError)) {
    throw new Test262Error("Expected TypeError, got " + e);
  }
}

verifyProperty(obj, "0", {
  enumerable: false,
  configurable: false,
});
