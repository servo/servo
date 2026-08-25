// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: When number absolute value is bigger of 2**1024 should convert to Infinity
es5id: 8.5_A14_T1
description: Create number bigger of 2**1024
---*/

//CHECK #1
if (1e+308*2 !== Infinity){
  throw new Test262Error('#1: 1e+308*2 === Infinity. Actual: ' + (1e+308*2));
}

//CHECK #2
if ((1*(Math.pow(2,53))*(Math.pow(2,971))) !== Infinity){
  throw new Test262Error('#2: (1*(Math.pow(2,53))*(Math.pow(2,971))) === Infinity. Actual: ' + ((1*(Math.pow(2,53))*(Math.pow(2,971)))));
}
