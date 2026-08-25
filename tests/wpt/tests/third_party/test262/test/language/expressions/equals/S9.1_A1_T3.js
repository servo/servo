// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    Result of primitive conversion from object is a default value for the
    Object
es5id: 9.1_A1_T3
description: >
    Using operator "+". This operator firstly calls ToPrimitive and
    then calls ToString or ToNumber
---*/

// CHECK#1
var object = {valueOf: function() {return 1}, toString: function() {return 0}};
if (object + "" !== "1") {
  throw new Test262Error('#1: var object = {valueOf: function() {return 1}, toString: function() {return 0}}; object + "" === "1". Actual: ' + (object + ""));
}

// CHECK#2
var object = {valueOf: function() {return "1"}, toString: function() {return 0}};
if (object + 0 !== "10") {
  throw new Test262Error('#2: var object = {valueOf: function() {return "1"}, toString: function() {return 0}}; object + 0 === "10". Actual: ' + (object + 0));
}
