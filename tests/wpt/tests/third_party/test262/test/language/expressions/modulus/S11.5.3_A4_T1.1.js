// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    The result of a ECMAScript floating-point remainder operation is
    determined by the rules of IEEE arithmetics
es5id: 11.5.3_A4_T1.1
description: If either operand is NaN, the result is NaN
---*/

//CHECK#1
if (isNaN(Number.NaN % Number.NaN) !== true) {
  throw new Test262Error('#1: NaN % NaN === Not-a-Number. Actual: ' + (NaN % NaN));
}  

//CHECK#2
if (isNaN(Number.NaN % +0) !== true) {
  throw new Test262Error('#2: NaN % +0 === Not-a-Number. Actual: ' + (NaN % +0)); 
} 

//CHECK#3
if (isNaN(Number.NaN % -0) !== true) {
  throw new Test262Error('#3: NaN % -0 === Not-a-Number. Actual: ' + (NaN % -0)); 
} 

//CHECK#4
if (isNaN(Number.NaN % Number.POSITIVE_INFINITY) !== true) {
  throw new Test262Error('#4: NaN % Infinity === Not-a-Number. Actual: ' + (NaN % Infinity));
} 

//CHECK#5
if (isNaN(Number.NaN % Number.NEGATIVE_INFINITY) !== true) {
  throw new Test262Error('#5: NaN % -Infinity === Not-a-Number. Actual: ' + (NaN % -Infinity)); 
} 

//CHECK#6
if (isNaN(Number.NaN % Number.MAX_VALUE) !== true) {
  throw new Test262Error('#6: NaN % Number.MAX_VALUE === Not-a-Number. Actual: ' + (NaN % Number.MAX_VALUE));
} 

//CHECK#7
if (isNaN(Number.NaN % Number.MIN_VALUE) !== true) {
  throw new Test262Error('#7: NaN % Number.MIN_VALUE === Not-a-Number. Actual: ' + (NaN % Number.MIN_VALUE)); 
}

//CHECK#8
if (isNaN(Number.NaN % 1) !== true) {
  throw new Test262Error('#8: NaN % 1 === Not-a-Number. Actual: ' + (NaN % 1));  
}
