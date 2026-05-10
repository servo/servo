// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    Type(x) and Type(y) are Boolean-s.
    Return true, if x and y are both true and both false; otherwise, return false
es5id: 11.9.4_A3
description: x and y are primitive booleans
---*/

//CHECK#1
if (!(true === true)) {
  throw new Test262Error('#1: true === true');
}

//CHECK#2
if (!(false === false)) {
  throw new Test262Error('#2: false === false');
}

//CHECK#3
if (true === false) {
  throw new Test262Error('#3: true !== false');
}

//CHECK#4
if (false === true) {
  throw new Test262Error('#4: false !== true');
}
