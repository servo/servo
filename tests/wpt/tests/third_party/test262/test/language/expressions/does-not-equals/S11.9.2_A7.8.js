// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    If Type(x) is Object and Type(y) is primitive type,
    return ToPrimitive(x) != y
es5id: 11.9.2_A7.8
description: x is object, y is primtitive
---*/

//CHECK#1
if ((true != {valueOf: function() {return 1}}) !== false) {
  throw new Test262Error('#1: (true != {valueOf: function() {return 1}}) === false');
}

//CHECK#2
if ((1 != {valueOf: function() {return 1}, toString: function() {return 0}}) !== false) {
  throw new Test262Error('#2: (1 != {valueOf: function() {return 1}, toString: function() {return 0}}) === false');
}

//CHECK#3
if (("+1" != {valueOf: function() {return 1}, toString: function() {return {}}}) !== false) {
  throw new Test262Error('#3: ("+1" != {valueOf: function() {return 1}, toString: function() {return {}}}) === false');
} 
  
//CHECK#4
try {
  if ((true != {valueOf: function() {return "+1"}, toString: function() {throw "error"}}) !== false) {
    throw new Test262Error('#4.1: (true != {valueOf: function() {return "+1"}, toString: function() {throw "error"}}) === false');
  }
}
catch (e) {
  if (e === "error") {
    throw new Test262Error('#4.2: (true != {valueOf: function() {return "+1"}, toString: function() {throw "error"}}) not throw "error"');
  } else {
    throw new Test262Error('#4.3: (true != {valueOf: function() {return "+1"}, toString: function() {throw "error"}}) not throw Error. Actual: ' + (e));
  }
}

//CHECK#5
if ((1 != {toString: function() {return "+1"}}) !== false) {
  throw new Test262Error('#5: (1 != {toString: function() {return "+1"}}) === false');
}

//CHECK#6
if (("1" != {valueOf: function() {return {}}, toString: function() {return "+1"}}) !== true) {
  throw new Test262Error('#6.1: ("1" != {valueOf: function() {return {}}, toString: function() {return "+1"}}) === true');
} else {
  if (("+1" != {valueOf: function() {return {}}, toString: function() {return "+1"}}) !== false) {
    throw new Test262Error('#6.2: ("+1" != {valueOf: function() {return {}}, toString: function() {return "+1"}}) === false');
  }
}

//CHECK#7
try {
  (1 != {valueOf: function() {throw "error"}, toString: function() {return 1}});
  throw new Test262Error('#7: (1 != {valueOf: function() {throw "error"}, toString: function() {return 1}}) throw "error"');
}  
catch (e) {
  if (e !== "error") {
    throw new Test262Error('#7: (1 != {valueOf: function() {throw "error"}, toString: function() {return 1}}) throw "error"');
  } 
}

//CHECK#8
try {
  (1 != {valueOf: function() {return {}}, toString: function() {return {}}});
  throw new Test262Error('#8: (1 != {valueOf: function() {return {}}, toString: function() {return {}}}) throw TypeError');
}  
catch (e) {
  if ((e instanceof TypeError) !== true) {
    throw new Test262Error('#8: (1 != {valueOf: function() {return {}}, toString: function() {return {}}}) throw TypeError');
  } 
}
