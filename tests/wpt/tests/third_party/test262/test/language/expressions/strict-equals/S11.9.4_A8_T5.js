// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: If Type(x) is different from Type(y), return false
es5id: 11.9.4_A8_T5
description: >
    Checking with such x and y that either x or y is primitive string
    and the other is primitive number
---*/

//CHECK#1
try {
  throw 1;
} catch(e) {
  if (e === "1") {
    throw new Test262Error('#1: throw 1 !== "1"');
  }
}

//CHECK#2
try {
  throw "1";
} catch(e) {
  if (1 === e) {
    throw new Test262Error('#2: 1 !== throw "1"');
  }
}
