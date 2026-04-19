// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: If ToBoolean(x) is true, return x
es5id: 11.11.2_A4_T1
description: >
    Type(x) and Type(y) vary between primitive boolean and Boolean
    object
---*/

//CHECK#1
if (((true || true)) !== true) {
  throw new Test262Error('#1: (true || true) === true');
}

//CHECK#2
if ((true || false) !== true) {
  throw new Test262Error('#2: (true || false) === true');
}

//CHECK#3
var x = new Boolean(true);
if ((x || new Boolean(true)) !== x) {
  throw new Test262Error('#3: (var x = new Boolean(true); (x || new Boolean(true)) === x');
}

//CHECK#4
var x = new Boolean(true);
if ((x || new Boolean(false)) !== x) {
  throw new Test262Error('#4: (var x = new Boolean(true); (x || new Boolean(false)) === x');
}

//CHECK#5
var x = new Boolean(false);
if ((x || new Boolean(true)) !== x) {
  throw new Test262Error('#5: (var x = new Boolean(false); (x || new Boolean(true)) === x');
}

//CHECK#6
var x = new Boolean(false);
if ((x || new Boolean(false)) !== x) {
  throw new Test262Error('#6: (var x = new Boolean(false); (x || new Boolean(false)) === x');
}
