// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: Check Break Statement for automatic semicolon insertion
es5id: 7.9_A2
description: Try use break \n Label construction
---*/

//CHECK#1
label1: for (var i = 0; i <= 0; i++) {
  for (var j = 0; j <= 0; j++) {
    break label1;
  }
  throw new Test262Error('#1: Check break statement for automatic semicolon insertion');
}

//CHECK#2
var result = false;
label2: for (var i = 0; i <= 0; i++) {
  for (var j = 0; j <= 0; j++) {
    break
    label2;
  }
  result = true;
}

if (result !== true) {
  throw new Test262Error('#2: Check break statement for automatic semicolon insertion');
}
