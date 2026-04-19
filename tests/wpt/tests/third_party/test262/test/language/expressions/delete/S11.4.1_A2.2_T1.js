// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: If GetBase(x) doesn't have a property GetPropertyName(x), return true
esid: sec-delete-operator-runtime-semantics-evaluation
description: Checking undeclared variable case
flags: [noStrict]
---*/

//CHECK#1
if (delete x !== true) {
  throw new Test262Error('#1: delete x === true');
}
