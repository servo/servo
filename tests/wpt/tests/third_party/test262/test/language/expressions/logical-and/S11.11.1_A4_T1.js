// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: If ToBoolean(x) is true, return y
es5id: 11.11.1_A4_T1
description: >
    Type(x) and Type(y) vary between primitive boolean and Boolean
    object
---*/

//CHECK#1
if ((true && true) !== true) {
  throw new Test262Error('#1: (true && true) === true');
}

//CHECK#2
if ((true && false) !== false) {
  throw new Test262Error('#2: (true && false) === false');
}

//CHECK#3
var y = new Boolean(true);
if ((new Boolean(true) &&  y) !== y) {
  throw new Test262Error('#3: (var y = new Boolean(true); (new Boolean(true) &&  y) === y');
}

//CHECK#4
var y = new Boolean(false);
if ((new Boolean(true) &&  y) !== y) {
  throw new Test262Error('#4: (var y = new Boolean(false); (new Boolean(true) &&  y) === y');
}

//CHECK#5
var y = new Boolean(true);
if ((new Boolean(false) &&  y) !== y) {
  throw new Test262Error('#5: (var y = new Boolean(true); (new Boolean(false) &&  y) === y');
}

//CHECK#6
var y = new Boolean(false);
if ((new Boolean(false) &&  y) !== y) {
  throw new Test262Error('#6: (var y = new Boolean(false); (new Boolean(false) &&  y) === y');
}
