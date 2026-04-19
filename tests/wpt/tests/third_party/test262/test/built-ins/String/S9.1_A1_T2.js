// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    Result of primitive conversion from object is a default value for the
    Object
es5id: 9.1_A1_T2
description: >
    Using operator Number. This operator calls ToPrimitive with hint
    Number
---*/

// CHECK#1
var object = {
  valueOf: function() {
    return 0
  },
  toString: function() {
    return 1
  }
};
if (String(object) !== "1") {
  throw new Test262Error('#1: var object = {valueOf: function() {return 0}, toString: function() {return 1}}; String(object) === "1". Actual: ' + (String(object)));
}

// CHECK#2
var object = {
  valueOf: function() {
    return 0
  },
  toString: function() {
    return {}
  }
};
if (String(object) !== "0") {
  throw new Test262Error('#2: var object = {valueOf: function() {return 0}, toString: function() {return {}}}; String(object) === "0". Actual: ' + (String(object)));
}
