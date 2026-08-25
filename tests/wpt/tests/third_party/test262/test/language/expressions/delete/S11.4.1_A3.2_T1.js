// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: If the property doesn't have the DontDelete attribute, return true
esid: sec-delete-operator-runtime-semantics-evaluation
description: Checking declared variable
flags: [noStrict]
---*/

//CHECK#1
x = 1;
if (delete x !== true) {
  throw new Test262Error('#1: x = 1; delete x === true');
}
