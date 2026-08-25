// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: Correct interpretation of multi line comments
es5id: 7.4_A2_T1
description: Create comments with any code
---*/

/*CHECK#1*/
/* Test262Error.thrower('#1: Correct interpretation multi line comments');
*/

/*CHECK#2*/
var x = 0;
/* x = 1;*/
assert.sameValue(x, 0, 'The value of `x` is 0');

//CHECK#3
var /* y = 1;*/
y;
assert.sameValue(y, undefined, 'The value of `y` is expected to equal `undefined`');

//CHECK#4
var /* y = 1;*/ y;
assert.sameValue(y, undefined, 'The value of `y` is expected to equal `undefined`');

/*CHECK#5*/
/*var x = 1;
if (x === 1) {
  Test262Error.thrower('#5: Correct interpretation multi line comments');
}
*/

/*CHECK#6*/
/*var this.y = 1;*/
this.y++;
assert.sameValue(isNaN(y), true, 'isNaN(y) returns true');

//CHECK#7
var string = "/*var y = 0*/" /* y = 1;*/ 
assert.sameValue(string, "/*var y = 0*/", 'The value of `string` is "/*var y = 0*/"');

//CHECK#8
var string = "/*var y = 0" /* y = 1;*/ 
assert.sameValue(string, "/*var y = 0", 'The value of `string` is "/*var y = 0"');

/*CHECK#9*/
/** Test262Error.thrower('#9: Correct interpretation multi line comments');
*/

/*CHECK#10*/
/* Test262Error.thrower('#10: Correct interpretation multi line comments');
**/

/*CHECK#11*/
/****** Test262Error.thrower('#11: Correct interpretation multi line comments');*********
***********
*


**********
**/
