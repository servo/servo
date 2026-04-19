// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    Result of number conversion from object value is the result
    of conversion from primitive value
es5id: 9.3_A5_T2
description: >
    new Number(), new Number(0), new Number(Number.NaN), new
    Number(null),  new Number(void 0) and others convert to Number by
    implicit transformation
---*/

// CHECK#1
if (+(new Number()) !== 0) {
  throw new Test262Error('#1: +(new Number()) === 0. Actual: ' + (+(new Number())));
}

// CHECK#2
if (+(new Number(0)) !== 0) {
  throw new Test262Error('#2: +(new Number(0)) === 0. Actual: ' + (+(new Number(0))));
}

// CHECK#3
if (isNaN(+(new Number(Number.NaN)) !== true)) {
  throw new Test262Error('#3: +(new Number(Number.NaN)) === Not-a-Number. Actual: ' + (+(new Number(Number.NaN))));
}

// CHECK#4
if (+(new Number(null)) !== 0) {
  throw new Test262Error('#4.1: +(new Number(null)) === 0. Actual: ' + (+(new Number(null)))); 
} else {
  if (1/+(new Number(null)) !== Number.POSITIVE_INFINITY) {
    throw new Test262Error('#4.2: +(new Number(null)) === +0. Actual: -0');
  }	
}

// CHECK#5
if (isNaN(+(new Number(void 0)) !== true)) {
  throw new Test262Error('#5: +(new Number(void 0)) === Not-a-Number. Actual: ' + (+(new Number(void 0))));
}

// CHECK#6
if (+(new Number(true)) !== 1) {
  throw new Test262Error('#6: +(new Number(true)) === 1. Actual: ' + (+(new Number(true))));
}

// CHECK#7
if (+(new Number(false)) !== +0) {
  throw new Test262Error('#7.1: +(new Number(false)) === 0. Actual: ' + (+(new Number(false))));
} else {
  if (1/+(new Number(false)) !== Number.POSITIVE_INFINITY) {
    throw new Test262Error('#7.2: +(new Number(false)) === +0. Actual: -0');
  }
}

// CHECK#8
if (+(new Boolean(true)) !== 1) {
  throw new Test262Error('#8: +(new Boolean(true)) === 1. Actual: ' + (+(new Boolean(true))));
}

// CHECK#9
if (+(new Boolean(false)) !== +0) {
  throw new Test262Error('#9.1: +(new Boolean(false)) === 0. Actual: ' + (+(new Boolean(false))));
} else {
  if (1/+(new Boolean(false)) !== Number.POSITIVE_INFINITY) {
    throw new Test262Error('#9.2: +(new Boolean(false)) === +0. Actual: -0');
  }
}

// CHECK#10
if (isNaN(+(new Array(2,4,8,16,32))) !== true) {
  throw new Test262Error('#10: +(new Array(2,4,8,16,32)) === Not-a-Number. Actual: ' + (+(new Array(2,4,8,16,32))));
}

// CHECK#11
var myobj1 = {
                ToNumber : function(){return 12345;}, 
                toString : function(){return "67890";},
                valueOf  : function(){return "[object MyObj]";} 
            };

if (isNaN(+(myobj1)) !== true){
  throw new Test262Error("#11: +(myobj1) calls ToPrimitive with hint +. Exptected: Not-a-Number. Actual: " + (+(myobj1)));
}

// CHECK#12
var myobj2 = {
                ToNumber : function(){return 12345;}, 
                toString : function(){return "67890";},
                valueOf  : function(){return "9876543210";} 
            };

if (+(myobj2) !== 9876543210){
  throw new Test262Error("#12: +(myobj2) calls ToPrimitive with hint +. Exptected: 9876543210. Actual: " + (+(myobj2)));
}


// CHECK#13
var myobj3 = {
                ToNumber : function(){return 12345;}, 
                toString : function(){return "[object MyObj]";} 
            };

if (isNaN(+(myobj3)) !== true){
  throw new Test262Error("#13: +(myobj3) calls ToPrimitive with hint +. Exptected: Not-a-Number. Actual: " + (+(myobj3)));
}

// CHECK#14
var myobj4 = {
                ToNumber : function(){return 12345;}, 
                toString : function(){return "67890";} 
            };

if (+(myobj4) !== 67890){
  throw new Test262Error("#14: +(myobj4) calls ToPrimitive with hint +. Exptected: 67890. Actual: " + (+(myobj4)));
}

// CHECK#15
var myobj5 = {
                ToNumber : function(){return 12345;} 
            };

if (isNaN(+(myobj5)) !== true){
  throw new Test262Error("#15: +(myobj5) calls ToPrimitive with hint +. Exptected: 12345. Actual: " + (+(myobj5)));
}
