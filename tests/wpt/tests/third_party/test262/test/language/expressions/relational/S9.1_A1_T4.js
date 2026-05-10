// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    Result of primitive conversion from object is a default value for the
    Object
es5id: 9.1_A1_T4
description: >
    Using operator "<". The operator firstly calls ToPrimitive and
    then calls ToString or ToNumber
---*/

// CHECK#1
var object = {valueOf: function() {return -2}, toString: function() {return "-2"}};
if ("-1" < object) {
  throw new Test262Error('#1: var object = {valueOf: function() {return -2}, toString: function() {return "-2"}}; "-1" < object');
}

// CHECK#2
var object = {valueOf: function() {return "-2"}, toString: function() {return -2}};
if (object < "-1") {
  throw new Test262Error('#2: var object = {valueOf: function() {return "-2"}, toString: function() {return -2}}; object < "-1"');
}
