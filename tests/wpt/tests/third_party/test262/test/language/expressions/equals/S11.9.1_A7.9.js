// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    If Type(x) is primitive type and Type(y) is Object,
    return x == ToPrimitive(y)
es5id: 11.9.1_A7.9
description: y is object, x is primtitive
---*/

//CHECK#1
if ((true == {valueOf: function() {return 1}}) !== true) {
  throw new Test262Error('#1: (true == {valueOf: function() {return 1}}) === true');
}

//CHECK#2
if ((1 == {valueOf: function() {return 1}, toString: function() {return 0}}) !== true) {
  throw new Test262Error('#2: (1 == {valueOf: function() {return 1}, toString: function() {return 0}}) === true');
}

//CHECK#3
if (("+1" == {valueOf: function() {return 1}, toString: function() {return {}}}) !== true) {
  throw new Test262Error('#3: ("+1" == {valueOf: function() {return 1}, toString: function() {return {}}}) === true');
} 
  
//CHECK#4
try {
  if ((true == {valueOf: function() {return "+1"}, toString: function() {throw "error"}}) !== true) {
    throw new Test262Error('#4.1: (true == {valueOf: function() {return "+1"}, toString: function() {throw "error"}}) === true');
  }
}
catch (e) {
  if (e === "error") {
    throw new Test262Error('#4.2: (true == {valueOf: function() {return "+1"}, toString: function() {throw "error"}}) not throw "error"');
  } else {
    throw new Test262Error('#4.3: (true == {valueOf: function() {return "+1"}, toString: function() {throw "error"}}) not throw Error. Actual: ' + (e));
  }
}

//CHECK#5
if ((1 == {toString: function() {return "+1"}}) !== true) {
  throw new Test262Error('#5: (1 == {toString: function() {return "+1"}}) === true');
}

//CHECK#6
if (("1" == {valueOf: function() {return {}}, toString: function() {return "+1"}}) !== false) {
  throw new Test262Error('#6.1: ("1" == {valueOf: function() {return {}}, toString: function() {return "+1"}}) === false');
} else {
  if (("+1" == {valueOf: function() {return {}}, toString: function() {return "+1"}}) !== true) {
    throw new Test262Error('#6.2: ("+1" == {valueOf: function() {return {}}, toString: function() {return "+1"}}) === true');
  }
}

//CHECK#7
try {
  (1 == {valueOf: function() {throw "error"}, toString: function() {return 1}});
  throw new Test262Error('#7.1: (1 == {valueOf: function() {throw "error"}, toString: function() {return 1}}) throw "error". Actual: ' + ((1 == {valueOf: function() {throw "error"}, toString: function() {return 1}})));
}  
catch (e) {
  if (e !== "error") {
    throw new Test262Error('#7.2: (1 == {valueOf: function() {throw "error"}, toString: function() {return 1}}) throw "error". Actual: ' + (e));
  } 
}

//CHECK#8
try {
  (1 == {valueOf: function() {return {}}, toString: function() {return {}}});
  throw new Test262Error('#8.1: (1 == {valueOf: function() {return {}}, toString: function() {return {}}}) throw TypeError. Actual: ' + ((1 == {valueOf: function() {return {}}, toString: function() {return {}}})));
}  
catch (e) {
  if ((e instanceof TypeError) !== true) {
    throw new Test262Error('#8.2: (1 == {valueOf: function() {return {}}, toString: function() {return {}}}) throw TypeError. Actual: ' + (e));
  } 
}
