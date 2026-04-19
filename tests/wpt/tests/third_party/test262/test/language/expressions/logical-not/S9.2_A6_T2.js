// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: Result of boolean conversion from object is true
es5id: 9.2_A6_T2
description: Different objects convert to Boolean by implicit transformation
---*/

// CHECK#1
if (!(new Object()) !== false) {
  throw new Test262Error('#1: !(new Object()) === false. Actual: ' + (!(new Object())));	
}

// CHECK#2
if (!(new String("")) !== false) {
  throw new Test262Error('#2: !(new String("")) === false. Actual: ' + (!(new String(""))));	
}

// CHECK#3
if (!(new String()) !== false) {
  throw new Test262Error('#3: !(new String()) === false. Actual: ' + (!(new String())));	
}

// CHECK#4
if (!(new Boolean(true)) !== false) {
  throw new Test262Error('#4: !(new Boolean(true)) === false. Actual: ' + (!(new Boolean(true))));	
}

// CHECK#5
if (!(new Boolean(false)) !== false) {
  throw new Test262Error('#5: !(new Boolean(false)) === false. Actual: ' + (!(new Boolean(false))));	
}

// CHECK#6
if (!(new Boolean()) !== false) {
  throw new Test262Error('#6: !(new Boolean()) === false. Actual: ' + (!(new Boolean())));	
}

// CHECK#7
if (!(new Array()) !== false) {
  throw new Test262Error('#7: !(new Array()) === false. Actual: ' + (!(new Array())));	
}

// CHECK#8
if (!(new Number()) !== false) {
  throw new Test262Error('#8: !(new Number()) === false. Actual: ' + (!(new Number())));	
}

// CHECK#9
if (!(new Number(-0)) !== false) {
  throw new Test262Error('#9: !(new Number(-0)) === false. Actual: ' + (!(new Number(-0))));	
}

// CHECK#10
if (!(new Number(0)) !== false) {
  throw new Test262Error('#10: !(new Number(0)) === false. Actual: ' + (!(new Number(0))));	
}

// CHECK#11
if (!(new Number()) !== false) {
  throw new Test262Error('#11: !(new Number()) === false. Actual: ' + (!(new Number())));	
}

// CHECK#12
if (!(new Number(Number.NaN)) !== false) {
  throw new Test262Error('#12: !(new Number(Number.NaN)) === false. Actual: ' + (!(new Number(Number.NaN))));	
}

// CHECK#13
if (!(new Number(-1)) !== false) {
  throw new Test262Error('#13: !(new Number(-1)) === false. Actual: ' + (!(new Number(-1))));	
}

// CHECK#14
if (!(new Number(1)) !== false) {
  throw new Test262Error('#14: !(new Number(1)) === false. Actual: ' + (!(new Number(1))));	
}

// CHECK#15
if (!(new Number(Number.POSITIVE_INFINITY)) !== false) {
  throw new Test262Error('#15: !(new Number(Number.POSITIVE_INFINITY)) === false. Actual: ' + (!(new Number(Number.POSITIVE_INFINITY))));	
}

// CHECK#16
if (!(new Number(Number.NEGATIVE_INFINITY)) !== false) {
  throw new Test262Error('#16: !(new Number(Number.NEGATIVE_INFINITY)) === false. Actual: ' + (!(new Number(Number.NEGATIVE_INFINITY))));	
}

// CHECK#17
if (!(new Function()) !== false) {
  throw new Test262Error('#17: !(new Function()) === false. Actual: ' + (!(new Function())));	
}

// CHECK#18
if (!(new Date()) !== false) {
  throw new Test262Error('#18: !(new Date()) === false. Actual: ' + (!(new Date())));	
}

// CHECK#19
if (!(new Date(0)) !== false) {
  throw new Test262Error('#19: !(new Date(0)) === false. Actual: ' + (!(new Date(0))));	
}
