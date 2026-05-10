// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    If Type(x) is primitive type and Type(y) is Object,
    return x != ToPrimitive(y)
es5id: 11.9.2_A7.9
description: y is object, x is primtitive
---*/

//CHECK#1
if (({valueOf: function() {return 1}} != true) !== false) {
  throw new Test262Error('#1: ({valueOf: function() {return 1}} != true) === false');
}

//CHECK#2
if (({valueOf: function() {return 1}, toString: function() {return 0}} != 1) !== false) {
  throw new Test262Error('#2: ({valueOf: function() {return 1}, toString: function() {return 0}} != 1) === false');
}

//CHECK#3
if (({valueOf: function() {return 1}, toString: function() {return {}}} != "+1") !== false) {
  throw new Test262Error('#3: ({valueOf: function() {return 1}, toString: function() {return {}}} != "+1") === false');
} 
  
//CHECK#4
try {
  if (({valueOf: function() {return "+1"}, toString: function() {throw "error"}} != true) !== false) {
    throw new Test262Error('#4.1: ({valueOf: function() {return "+1"}, toString: function() {throw "error"}} != true) === false');
  }
}
catch (e) {
  if (e === "error") {
    throw new Test262Error('#4.2: ({valueOf: function() {return "+1"}, toString: function() {throw "error"}} != true) not throw "error"');
  } else {
    throw new Test262Error('#4.3: ({valueOf: function() {return "+1"}, toString: function() {throw "error"}} != true) not throw Error. Actual: ' + (e));
  }
}

//CHECK#5
if (({toString: function() {return "+1"}} != 1) !== false) {
  throw new Test262Error('#5: ({toString: function() {return "+1"}} != 1) === false');
}

//CHECK#6
if (({valueOf: function() {return {}}, toString: function() {return "+1"}} != "1") !== true) {
  throw new Test262Error('#6.1: ({valueOf: function() {return {}}, toString: function() {return "+1"}} != "1") === true');
} else {
  if (({valueOf: function() {return {}}, toString: function() {return "+1"}} != "+1") !== false) {
    throw new Test262Error('#6.2: ({valueOf: function() {return {}}, toString: function() {return "+1"}} != "+1") === false');
  }
}

//CHECK#7
try {
  ({valueOf: function() {throw "error"}, toString: function() {return 1}} != 1);
  throw new Test262Error('#7.1: ({valueOf: function() {throw "error"}, toString: function() {return 1}} != 1) throw "error". Actual: ' + (({valueOf: function() {throw "error"}, toString: function() {return 1}} != 1)));
}  
catch (e) {
  if (e !== "error") {
    throw new Test262Error('#7.2: ({valueOf: function() {throw "error"}, toString: function() {return 1}} != 1) throw "error". Actual: ' + (e));
  } 
}

//CHECK#8
try {
  ({valueOf: function() {return {}}, toString: function() {return {}}} != 1);
  throw new Test262Error('#8.1: ({valueOf: function() {return {}}, toString: function() {return {}}} != 1) throw TypeError. Actual: ' + (({valueOf: function() {return {}}, toString: function() {return {}}} != 1)));
}  
catch (e) {
  if ((e instanceof TypeError) !== true) {
    throw new Test262Error('#8.2: ({valueOf: function() {return {}}, toString: function() {return {}}} != 1) throw TypeError. Actual: ' + (e));
  } 
}
