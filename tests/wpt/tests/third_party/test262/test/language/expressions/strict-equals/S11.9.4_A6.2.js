// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: If Type(x) and Type(y) are Null-s, return true
es5id: 11.9.4_A6.2
description: null === null
---*/

//CHECK#1
if (!(null === null)) {
  throw new Test262Error('#1: null === null');
}
