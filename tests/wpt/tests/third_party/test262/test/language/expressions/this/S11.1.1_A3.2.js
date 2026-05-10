// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    Being in function code, "this" and eval("this"), called as a
    constructors, return the object
es5id: 11.1.1_A3.2
description: Create function. It have property, that returned "this"
---*/

//CHECK#1
function SetThis() {this.THIS = this}
if ((new SetThis()).THIS.toString() !== "[object Object]") {
  throw new Test262Error('#1: function SetThis() {this.THIS = this} (new SetThis()).THIS.toString() !== "[object Object]". Actual: ' + ((new SetThis()).THIS.toString()));
}

//CHECK#2
function SetEvalThis() {this.THIS = eval("this")}
if ((new SetEvalThis()).THIS.toString() !== "[object Object]") {
  throw new Test262Error('#2: function SetEvalThis() {this.THIS = eval("this")} (new SetEvalThis()).THIS.toString() !== "[object Object]". Actual: ' + ((new SetEvalThis()).THIS.toString()));
}
