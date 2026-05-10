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
MyObjectNotVar = new MyFunction();
MyObjectNotVar.prop = 1;
delete MyObjectNotVar.prop;
if (MyObjectNotVar.prop !== undefined) {
  throw new Test262Error(
    '#1: function MyFunction(){}; MyObjectNotVar = new MyFunction(); MyFunction.prop = 1; delete MyObjectNotVar.prop; MyObjectNotVar.prop === undefined. Actual: ' +
    MyObjectNotVar.prop
  );
}
