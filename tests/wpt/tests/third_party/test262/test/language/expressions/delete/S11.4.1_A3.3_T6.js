// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: If the property doesn't have the DontDelete attribute, remove the property
esid: sec-delete-operator-runtime-semantics-evaluation
description: Checking declared variable
flags: [noStrict]
---*/

//CHECK#1
function MyFunction() {}
var MyObjectVar = new MyFunction();
if (delete MyObjectNotVar !== true) {
  throw new Test262Error(
    '#1: function MyFunction(){}; var MyObjectNotVar = new MyFunction(); delete MyObjectNotVar === true'
  );
}
