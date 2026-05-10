// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: Operator x++ uses [[Default Value]]
es5id: 11.3.1_A2.2_T1
description: If Type(value) is Object, evaluate ToPrimitive(value, Number)
---*/

//CHECK#1
var object = {valueOf: function() {return 1}};
var y = object++;
if (y !== 1) {
  throw new Test262Error('#1: var object = {valueOf: function() {return 1}}; var y = object++; y === 1. Actual: ' + (y));
} else {
  if (object !== 1 + 1) {
    throw new Test262Error('#1: var object = {valueOf: function() {return 1}}; object++; object === 1 + 1. Actual: ' + (object));
  }
}

//CHECK#2
var object = {valueOf: function() {return 1}, toString: function() {return 0}};
var y = object++;
if (y !== 1) {
  throw new Test262Error('#2: var object = {valueOf: function() {return 1}, toString: function() {return 0}}; var y = object++; y === 1. Actual: ' + (y));
} else {
  if (object !== 1 + 1) {
    throw new Test262Error('#2: var object = {valueOf: function() {return 1}, toString: function() {return 0}}; object++; object === 1 + 1. Actual: ' + (object));
  }
}

//CHECK#3
var object = {valueOf: function() {return 1}, toString: function() {return {}}};
var y = object++;
if (y !== 1) {
  throw new Test262Error('#3: var object = {valueOf: function() {return 1}, toString: function() {return {}}}; var y = object++; y === 1. Actual: ' + (y));
} else {
  if (object !== 1 + 1) {
    throw new Test262Error('#3: var object = {valueOf: function() {return 1}, toString: function() {return {}}}; object++; object === 1 + 1. Actual: ' + (object));
  }
}

//CHECK#4
try {
  var object = {valueOf: function() {return 1}, toString: function() {throw "error"}};
  var y = object++;
  if (y !== 1) {
    throw new Test262Error('#4.1: var object = {valueOf: function() {return 1}, toString: function() {throw "error"}}; var y = object++; y === 1. Actual: ' + (y));
  } else {
    if (object !== 1 + 1) {
      throw new Test262Error('#4.2: var object = {valueOf: function() {return 1}, toString: function() {throw "error"}}; object++; object === 1 + 1. Actual: ' + (object));
    }
  }
}
catch (e) {
  if (e === "error") {
    throw new Test262Error('#4.3: var object = {valueOf: function() {return 1}, toString: function() {throw "error"}}; var y = object++; y not throw "error"');
  } else {
    throw new Test262Error('#4.4: var object = {valueOf: function() {return 1}, toString: function() {throw "error"}}; var y = object++; y not throw Error. Actual: ' + (e));
  }
}

//CHECK#5
var object = {toString: function() {return 1}};
var y = object++;
if (y !== 1) {
  throw new Test262Error('#5.1: var object = {toString: function() {return 1}}; var y = object++; y === 1. Actual: ' + (y));
}  else {
  if (object !== 1 + 1) {
    throw new Test262Error('#5.2: var object = {toString: function() {return 1}}; object++; object === 1 + 1. Actual: ' + (object));
  }
}


//CHECK#6
var object = {valueOf: function() {return {}}, toString: function() {return 1}}
var y = object++;
if (y !== 1) {
  throw new Test262Error('#6.1: var object = {valueOf: function() {return {}}, toString: function() {return 1}}; var y = object++; y === 1. Actual: ' + (y));
} else {
  if (object !== 1 + 1) {
    throw new Test262Error('#6.2: var object = {valueOf: function() {return {}}, toString: function() {return 1}}; object++; object === 1 + 1. Actual: ' + (object));
  }
}

//CHECK#7
try {
  var object = {valueOf: function() {throw "error"}, toString: function() {return 1}};
  var y = object++;
  throw new Test262Error('#7.1: var object = {valueOf: function() {throw "error"}, toString: function() {return 1}}; object++ throw "error". Actual: ' + (y));
}  
catch (e) {
  if (e !== "error") {
    throw new Test262Error('#7.2: var object = {valueOf: function() {throw "error"}, toString: function() {return 1}}; object++ throw "error". Actual: ' + (e));
  } 
}

//CHECK#8
try {
  var object = {valueOf: function() {return {}}, toString: function() {return {}}};
  var y = object++;
  throw new Test262Error('#8.1: var object = {valueOf: function() {return {}}, toString: function() {return {}}}; object++ throw TypeError. Actual: ' + (y));
}  
catch (e) {
  if ((e instanceof TypeError) !== true) {
    throw new Test262Error('#8.2: var object = {valueOf: function() {return {}}, toString: function() {return {}}}; object++ throw TypeError. Actual: ' + (e));
  } 
}
