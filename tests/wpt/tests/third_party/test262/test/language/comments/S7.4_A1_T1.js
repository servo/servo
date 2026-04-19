// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: Correct interpretation of single line comments
es5id: 7.4_A1_T1
description: Create comments with any code
---*/

//CHECK#1
// Test262Error.thrower('#1: Correct interpretation single line comments');

//CHECK#2
var x = 0;
assert.sameValue(x, 0, 'The value of `x` is 0');

//CHECK#3
var // y = 1; 
y;
assert.sameValue(y, undefined, 'The value of `y` is expected to equal `undefined`');

//CHECK#4
//Test262Error.thrower('#4: Correct interpretation single line comments') //Test262Error.thrower('#4: Correct interpretation single line comments'); //

////CHECK#5
//var x = 1;
//if (x === 1) {
//  Test262Error.thrower('#5: Correct interpretation single line comments');
//}

//CHECK#6
//var this.y = 1; 
this.y++;
assert.sameValue(isNaN(y), true, 'isNaN(y) returns true');
