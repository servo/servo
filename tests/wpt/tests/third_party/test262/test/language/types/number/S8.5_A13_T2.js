// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    Finite nonzero values  that are Normalised having the form s*m*2**e
    where s is +1 or -1, m is a positive integer less than 2**53 but not
    less than s**52 and e is an integer ranging from -1074 to 971
es5id: 8.5_A13_T2
description: Finite Non zero values where e is 971
---*/

//CHECK #1
if ((1*(Math.pow(2,52))*(Math.pow(2,971))) !== 8.98846567431158e+307){
  throw new Test262Error('#1: (1*(Math.pow(2,52))*(Math.pow(2,971))) === 8.98846567431158e+307. Actual: ' + ((1*(Math.pow(2,52))*(Math.pow(2,971)))));
}

//CHECK #2
if ((1*((Math.pow(2,53))-1)*(Math.pow(2,971))) !== 1.7976931348623157e+308){
  throw new Test262Error('#2: (1*((Math.pow(2,53))-1)*(Math.pow(2,971))) === 1.7976931348623157e+308. Actual: ' + ((1*((Math.pow(2,53))-1)*(Math.pow(2,971)))));
}

//CHECK #3
if ((-1*(Math.pow(2,52))*(Math.pow(2,971))) !== -8.98846567431158e+307){
  throw new Test262Error('#3: (-1*(Math.pow(2,52))*(Math.pow(2,971))) === -8.98846567431158e+307. Actual: ' + ((-1*(Math.pow(2,52))*(Math.pow(2,971)))));
}
