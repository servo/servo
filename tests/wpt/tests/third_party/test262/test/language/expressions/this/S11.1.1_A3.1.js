// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    Being in function code, "this" and eval("this"), called as a functions,
    return the global object
es5id: 11.1.1_A3.1
description: Creating function which returns "this" or eval("this")
flags: [noStrict]
---*/

//CHECK#1
function ReturnThis() {return this}
if (ReturnThis() !== this) {
  throw new Test262Error('#1: function ReturnThis() {return this} ReturnThis() === this. Actual: ' + (ReturnThis()));
}

//CHECK#2
function ReturnEvalThis() {return eval("this")}
if (ReturnEvalThis() !== this) {
  throw new Test262Error('#2: function ReturnEvalThis() {return eval("this")} ReturnEvalThis() === this. Actual: ' + (ReturnEvalThis()));
}
