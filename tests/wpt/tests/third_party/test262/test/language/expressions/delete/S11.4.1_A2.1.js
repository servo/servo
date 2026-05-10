// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: If Type(x) is not Reference, return true
esid: sec-delete-operator-runtime-semantics-evaluation
description: Checking primitive value and Object value cases
---*/

//CHECK#1
if (delete 1 !== true) {
  throw new Test262Error('#1: delete 1 === true');
}

//CHECK#2
if (delete new Object() !== true) {
  throw new Test262Error('#2: delete new Object() === true');
}
