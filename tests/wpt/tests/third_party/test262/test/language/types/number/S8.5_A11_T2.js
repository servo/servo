// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: The integer 0 has two representations, +0 and -0
es5id: 8.5_A11_T2
description: Compare positive_zero and negative_zero
---*/

var p_zero=+0;
var n_zero=-0;

//CHECK #1
if ((p_zero == n_zero) !== true){
  throw new Test262Error('#1: var p_zero=+0; var n_zero=-0; p_zero != n_zero');
}

//CHECK #2
if ((n_zero == 0) !== true){
  throw new Test262Error('#2: var p_zero=+0; var n_zero=-0; n_zero == 0');
}

//CHECK #3
if ((p_zero == -0) !== true){
  throw new Test262Error('#3: var p_zero=+0; var n_zero=-0; p_zero == -0');
}

//CHECK #4
if ((p_zero === 0) !== true){
  throw new Test262Error('#4: var p_zero=+0; var n_zero=-0; p_zero === 0');
}

//CHECK #5
if ((n_zero === -0) !== true){
  throw new Test262Error('#5: var p_zero=+0; var n_zero=-0; n_zero === -0');
}
