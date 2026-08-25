// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: If the property doesn't have the DontDelete attribute, remove the property
esid: sec-delete-operator-runtime-semantics-evaluation
description: Checking declared variable
---*/

//CHECK#1
function MyFunction() {}
MyFunction.prop = 1;
delete MyFunction.prop;
if (MyFunction.prop !== undefined) {
  throw new Test262Error(
    '#1: function MyFunction(){}; MyFunction.prop = 1; delete MyFunction.prop; MyFunction.prop === undefined. Actual: ' +
    MyFunction.prop
  );
}
