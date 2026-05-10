// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: Operator !x uses GetValue
es5id: 11.4.9_A2.1_T1
description: Either Type(x) is not Reference or GetBase(x) is not null
---*/

//CHECK#1
if (!true !== false) {
  throw new Test262Error('#1: !true === false');
}

//CHECK#2
if (!(!true) !== true) {
  throw new Test262Error('#2: !(!true) === true');
}

//CHECK#3
var x = true;
if (!x !== false) {
  throw new Test262Error('#3: var x = true; !x === false');
}

//CHECK#4
var x = true;
if (!(!x) !== true) {
  throw new Test262Error('#4: var x = true; !(!x) === true');
}

//CHECK#5
var object = new Object();
object.prop = true;
if (!object.prop !== false) {
  throw new Test262Error('#5: var object = new Object(); object.prop = true; !object.prop === false');
}
