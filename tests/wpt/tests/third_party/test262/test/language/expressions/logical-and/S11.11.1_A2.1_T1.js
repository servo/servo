// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: Operator x && y uses GetValue
es5id: 11.11.1_A2.1_T1
description: Either Type is not Reference or GetBase is not null
---*/

//CHECK#1
if ((false && true) !== false) {
  throw new Test262Error('#1: (false && true) === false');
}

//CHECK#2
if ((true && false) !== false) {
  throw new Test262Error('#2: (true && false) === false');
}

//CHECK#3
var x = false;
if ((x && true) !== false) {
  throw new Test262Error('#3: var x = false; (x && true) === false');
}

//CHECK#4
var y = new Boolean(false);
if ((true && y) !== y) {
  throw new Test262Error('#4: var y = new Boolean(false); (true && y) === y');
}

//CHECK#5
var x = false;
var y = true;
if ((x && y) !== false) {
  throw new Test262Error('#5: var x = false; var y = true; (x && y) === false');
}

//CHECK#6
var x = true;
var y = new Boolean(false);
if ((x && y) !== y) {
  throw new Test262Error('#6: var x = true; var y = new Boolean(false); (x && y) === y');
}

//CHECK#7
var objectx = new Object();
var objecty = new Object();
objectx.prop = true;
objecty.prop = 1.1;
if ((objectx.prop && objecty.prop) !== objecty.prop) {
  throw new Test262Error('#7: var objectx = new Object(); var objecty = new Object(); objectx.prop = true; objecty.prop = 1; (objectx.prop && objecty.prop) === objecty.prop');
}

//CHECK#8
var objectx = new Object();
var objecty = new Object();
objectx.prop = 0;
objecty.prop = true;
if ((objectx.prop && objecty.prop) !== objectx.prop) {
  throw new Test262Error('#8: var objectx = new Object(); var objecty = new Object(); objectx.prop = 0; objecty.prop = true; (objectx.prop && objecty.prop) === objectx.prop');
}
