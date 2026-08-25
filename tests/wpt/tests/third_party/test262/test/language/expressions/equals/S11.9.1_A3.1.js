// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    Return true, if x and y are both true or both false; otherwise, return
    false
es5id: 11.9.1_A3.1
description: x and y are boolean primitives
---*/

//CHECK#1
if ((true == true) !== true) {
  throw new Test262Error('#1: (true == true) === true');
}

//CHECK#2
if ((false == false) !== true) {
  throw new Test262Error('#2: (false == false) === true');
}

//CHECK#3
if ((true == false) !== false) {
  throw new Test262Error('#3: (true == false) === false');
}

//CHECK#4
if ((false == true) !== false) {
  throw new Test262Error('#4: (false == true) === false');
}
