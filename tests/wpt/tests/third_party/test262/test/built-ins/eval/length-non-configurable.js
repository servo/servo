// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: The length property of eval does not have the attribute DontDelete
esid: sec-eval-x
description: Checking use hasOwnProperty, delete
---*/

//CHECK#1
if (eval.hasOwnProperty('length') !== true) {
  throw new Test262Error('#1: eval.hasOwnProperty(\'length\') === true. Actual: ' + (eval.hasOwnProperty('length')));
}

delete eval.length;

//CHECK#2
if (eval.hasOwnProperty('length') !== false) {
  throw new Test262Error('#2: delete eval.length; eval.hasOwnProperty(\'length\') === false. Actual: ' + (eval.hasOwnProperty('length')));
}

//CHECK#3
if (eval.length === undefined) {
  throw new Test262Error('#3: delete eval.length; eval.length !== undefined');
}
